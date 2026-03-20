-- ==============================================================
-- Actuators — SurrealDB Schema
-- نظام التشغيل الاقتصادي للشركات الناشئة والفرق التطوعية
-- ==============================================================

-- ============================================================
-- NAMESPACE & DATABASE
-- ============================================================
DEFINE NAMESPACE Actuators;
USE NS Actuators;
DEFINE DATABASE erp;
USE DB erp;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١: الهوية والصلاحيات (Identity & RBAC)           ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: account  (الهوية الموحدة - RBAC)
-- كل شخص له سجل account واحد.
-- org_admin = مسؤول شركة ناشئة أو فريق تطوعي (يدير فريقه ومشاريعه)
-- org_member = عضو في منظمة (يرى مشاريع منظمته فقط)
-- ============================================================
DEFINE TABLE account SCHEMAFULL
    PERMISSIONS
        FOR select WHERE id = $auth.id OR $auth.role = "admin"
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD email         ON account TYPE string   ASSERT string::is::email($value);
DEFINE FIELD password_hash ON account TYPE string;
DEFINE FIELD role          ON account TYPE string
    ASSERT $value IN ["admin", "manager", "employee", "org_admin", "org_member", "intern", "applicant"]
    VALUE $value OR "applicant";
DEFINE FIELD is_active     ON account TYPE bool     VALUE $value OR true;
DEFINE FIELD auth_methods  ON account TYPE array    VALUE $value OR [];
DEFINE FIELD avatar        ON account TYPE option<string>;
DEFINE FIELD created_at    ON account TYPE datetime VALUE $value OR time::now();
DEFINE FIELD last_login    ON account TYPE option<datetime>;

DEFINE INDEX idx_account_email ON account FIELDS email UNIQUE;

-- ============================================================
-- TABLE: profile  (السيرة الذاتية والمهارات)
-- ============================================================
DEFINE TABLE profile SCHEMAFULL
    PERMISSIONS
        FOR select WHERE account = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR update WHERE account = $auth.id OR $auth.role = "admin"
        FOR create, delete WHERE $auth.role = "admin";

DEFINE FIELD account      ON profile TYPE record<account>;
DEFINE FIELD full_name    ON profile TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD national_id  ON profile TYPE string;
DEFINE FIELD phone        ON profile TYPE string;
DEFINE FIELD address      ON profile TYPE string;
DEFINE FIELD birth_date   ON profile TYPE datetime;
DEFINE FIELD gender       ON profile TYPE string   ASSERT $value IN ["male", "female"];
DEFINE FIELD skills       ON profile TYPE array    VALUE $value OR [];
DEFINE FIELD education    ON profile TYPE array    VALUE $value OR [];
DEFINE FIELD resume_url   ON profile TYPE option<string>;
DEFINE FIELD photo_url    ON profile TYPE option<string>;
DEFINE FIELD metadata     ON profile TYPE object   VALUE $value OR {};

DEFINE INDEX idx_profile_account ON profile FIELDS account UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٢: المنظمات (الشركات الناشئة والفرق التطوعية)     ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: organization  (الكيان — شركة ناشئة أو فريق تطوعي)
-- كل كيان يسجل في المنصة ويحصل على لوحة تحكم خاصة
-- ============================================================
DEFINE TABLE organization SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
            OR id IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id)
        FOR create WHERE $auth.role IN ["admin", "manager", "org_admin"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
            OR id IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role = "owner")
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD name          ON organization TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD name_en       ON organization TYPE option<string>;
DEFINE FIELD org_type      ON organization TYPE string
    ASSERT $value IN ["startup", "volunteer_team"]
    VALUE $value OR "startup";
DEFINE FIELD description   ON organization TYPE option<string>;
DEFINE FIELD logo_url      ON organization TYPE option<string>;
DEFINE FIELD sector        ON organization TYPE option<string>;
-- التصنيف الفرعي: مجال الشركة أو نوع الفريق التطوعي
DEFINE FIELD tags          ON organization TYPE array    VALUE $value OR [];
DEFINE FIELD email         ON organization TYPE option<string>;
DEFINE FIELD phone         ON organization TYPE option<string>;
DEFINE FIELD website       ON organization TYPE option<string>;
DEFINE FIELD city          ON organization TYPE option<string>;
DEFINE FIELD country       ON organization TYPE string   VALUE $value OR "SY";
DEFINE FIELD founded_date  ON organization TYPE option<datetime>;
-- عدد أعضاء الفريق (محدّث تلقائياً أو يدوياً)
DEFINE FIELD team_size     ON organization TYPE int      VALUE $value OR 1;
DEFINE FIELD stage         ON organization TYPE string
    ASSERT $value IN ["idea", "mvp", "growth", "scaling", "active", "inactive"]
    VALUE $value OR "idea";
DEFINE FIELD status        ON organization TYPE string
    ASSERT $value IN ["pending", "approved", "suspended", "archived"]
    VALUE $value OR "pending";
DEFINE FIELD social_links  ON organization TYPE object   VALUE $value OR {};
DEFINE FIELD metadata      ON organization TYPE object   VALUE $value OR {};
DEFINE FIELD created_at    ON organization TYPE datetime VALUE $value OR time::now();
DEFINE FIELD updated_at    ON organization TYPE datetime VALUE time::now();

DEFINE INDEX idx_org_name   ON organization FIELDS name;
DEFINE INDEX idx_org_type   ON organization FIELDS org_type;
DEFINE INDEX idx_org_status ON organization FIELDS status;

-- ============================================================
-- TABLE: org_membership  (عضوية الحسابات في المنظمات)
-- يربط الحسابات بالمنظمات مع تحديد الدور
-- owner = مؤسس المنظمة (له كامل الصلاحيات)
-- admin = مسؤول (يدير الأعضاء والمشاريع)
-- member = عضو عادي
-- ============================================================
DEFINE TABLE org_membership SCHEMAFULL
    PERMISSIONS
        FOR select WHERE account = $auth.id OR $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id)
        FOR create WHERE $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role IN ["owner", "admin"])
        FOR update WHERE $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role IN ["owner", "admin"])
        FOR delete WHERE $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role = "owner");

DEFINE FIELD account       ON org_membership TYPE record<account>;
DEFINE FIELD organization  ON org_membership TYPE record<organization>;
DEFINE FIELD role          ON org_membership TYPE string
    ASSERT $value IN ["owner", "admin", "member"]
    VALUE $value OR "member";
DEFINE FIELD title         ON org_membership TYPE option<string>;
DEFINE FIELD joined_at     ON org_membership TYPE datetime VALUE $value OR time::now();
DEFINE FIELD is_active     ON org_membership TYPE bool VALUE $value OR true;

DEFINE INDEX idx_orgmem_unique ON org_membership FIELDS account, organization UNIQUE;
DEFINE INDEX idx_orgmem_org    ON org_membership FIELDS organization;
DEFINE INDEX idx_orgmem_acct   ON org_membership FIELDS account;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٣: الفعاليات (Events)                             ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: event  (الفعاليات والأنشطة)
-- ورش عمل، هاكاثونات، لقاءات، تدريبات، معارض
-- ============================================================
DEFINE TABLE event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true  -- الفعاليات عامة للقراءة
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD title          ON event TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD title_en       ON event TYPE option<string>;
DEFINE FIELD slug           ON event TYPE option<string>;
DEFINE FIELD description    ON event TYPE option<string>;
DEFINE FIELD event_type     ON event TYPE string
    ASSERT $value IN ["workshop", "hackathon", "meetup", "training", "conference", "exhibition", "networking", "demo_day", "other"]
    VALUE $value OR "meetup";
DEFINE FIELD cover_url      ON event TYPE option<string>;
-- الموقع: فعلي أو أونلاين
DEFINE FIELD location       ON event TYPE option<string>;
DEFINE FIELD location_type  ON event TYPE string
    ASSERT $value IN ["onsite", "online", "hybrid"]
    VALUE $value OR "onsite";
DEFINE FIELD online_url     ON event TYPE option<string>;
DEFINE FIELD city           ON event TYPE option<string>;
DEFINE FIELD start_date     ON event TYPE datetime;
DEFINE FIELD end_date       ON event TYPE datetime;
DEFINE FIELD capacity       ON event TYPE option<int>;
DEFINE FIELD registered_count ON event TYPE int VALUE $value OR 0;
-- هل الفعالية مجانية أم مدفوعة؟
DEFINE FIELD is_free        ON event TYPE bool VALUE $value OR true;
DEFINE FIELD price          ON event TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD currency       ON event TYPE string VALUE $value OR "SYP";
DEFINE FIELD organizer      ON event TYPE option<record<organization>>;
DEFINE FIELD organized_by   ON event TYPE option<record<account>>;
DEFINE FIELD speakers       ON event TYPE array VALUE $value OR [];
-- [{name, title, photo_url, bio}]
DEFINE FIELD agenda         ON event TYPE array VALUE $value OR [];
-- [{time, title, description, speaker}]
DEFINE FIELD tags           ON event TYPE array VALUE $value OR [];
DEFINE FIELD status         ON event TYPE string
    ASSERT $value IN ["draft", "published", "ongoing", "completed", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD metadata       ON event TYPE object VALUE $value OR {};
DEFINE FIELD created_at     ON event TYPE datetime VALUE $value OR time::now();
DEFINE FIELD updated_at     ON event TYPE datetime VALUE time::now();

DEFINE INDEX idx_event_slug   ON event FIELDS slug UNIQUE;
DEFINE INDEX idx_event_type   ON event FIELDS event_type;
DEFINE INDEX idx_event_status ON event FIELDS status;
DEFINE INDEX idx_event_date   ON event FIELDS start_date;

-- ============================================================
-- TABLE: event_registration  (تسجيل الحضور في الفعاليات)
-- يمكن لأفراد أو منظمات التسجيل
-- ============================================================
DEFINE TABLE event_registration SCHEMAFULL
    PERMISSIONS
        FOR select WHERE account = $auth.id OR $auth.role IN ["admin", "manager"]
            OR event IN (SELECT VALUE id FROM event)
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD event          ON event_registration TYPE record<event>;
DEFINE FIELD account        ON event_registration TYPE record<account>;
DEFINE FIELD organization   ON event_registration TYPE option<record<organization>>;
DEFINE FIELD status         ON event_registration TYPE string
    ASSERT $value IN ["registered", "confirmed", "attended", "no_show", "cancelled"]
    VALUE $value OR "registered";
DEFINE FIELD ticket_type    ON event_registration TYPE string
    ASSERT $value IN ["individual", "team", "speaker", "sponsor", "volunteer"]
    VALUE $value OR "individual";
DEFINE FIELD notes          ON event_registration TYPE option<string>;
DEFINE FIELD checked_in_at  ON event_registration TYPE option<datetime>;
DEFINE FIELD registered_at  ON event_registration TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_eventreg_unique ON event_registration FIELDS event, account UNIQUE;
DEFINE INDEX idx_eventreg_event  ON event_registration FIELDS event;
DEFINE INDEX idx_eventreg_org    ON event_registration FIELDS organization;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٤: الموارد البشرية (HR)                          ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: employment_contract  (عقد العمل — لموظفي Actuators)
-- ============================================================
DEFINE TABLE employment_contract SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee          ON employment_contract TYPE record<account>;
DEFINE FIELD position          ON employment_contract TYPE string;
DEFINE FIELD department        ON employment_contract TYPE string;
DEFINE FIELD employment_type   ON employment_contract TYPE string
    ASSERT $value IN ["full_time", "part_time", "intern", "contract"];
DEFINE FIELD start_date        ON employment_contract TYPE datetime;
DEFINE FIELD end_date          ON employment_contract TYPE option<datetime>;
DEFINE FIELD weekly_hours      ON employment_contract TYPE int
    ASSERT $value > 0 AND $value <= 168;
DEFINE FIELD transport_allowance ON employment_contract TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD work_location     ON employment_contract TYPE string
    ASSERT $value IN ["onsite", "remote", "hybrid"];
DEFINE FIELD is_active         ON employment_contract TYPE bool  VALUE $value OR true;
DEFINE FIELD created_at        ON employment_contract TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: salary  (الرواتب — Admin فقط)
-- ============================================================
DEFINE TABLE salary SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee       ON salary TYPE record<account>;
DEFINE FIELD base_amount    ON salary TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD currency       ON salary TYPE string   VALUE $value OR "SYP";
DEFINE FIELD effective_from ON salary TYPE datetime;
DEFINE FIELD effective_to   ON salary TYPE option<datetime>;
DEFINE FIELD notes          ON salary TYPE option<string>;

-- ============================================================
-- TABLE: payroll  (كشف الرواتب الشهري)
-- ============================================================
DEFINE TABLE payroll SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee        ON payroll TYPE record<account>;
DEFINE FIELD period_month    ON payroll TYPE int    ASSERT $value >= 1 AND $value <= 12;
DEFINE FIELD period_year     ON payroll TYPE int    ASSERT $value >= 2020;
DEFINE FIELD base_salary     ON payroll TYPE decimal ASSERT $value >= 0;
DEFINE FIELD housing_allowance ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD transport_allowance ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD other_allowances ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD overtime_hours  ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD overtime_amount ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD deductions      ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD deduction_notes ON payroll TYPE option<string>;
DEFINE FIELD net_salary      ON payroll TYPE decimal VALUE $value OR (base_salary + housing_allowance + transport_allowance + other_allowances + overtime_amount - deductions);
DEFINE FIELD currency        ON payroll TYPE string  VALUE $value OR "SYP";
DEFINE FIELD payment_method  ON payroll TYPE string
    ASSERT $value IN ["bank_transfer", "cash", "check"]
    VALUE $value OR "bank_transfer";
DEFINE FIELD payment_date    ON payroll TYPE option<datetime>;
DEFINE FIELD status          ON payroll TYPE string
    ASSERT $value IN ["draft", "approved", "paid", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD approved_by     ON payroll TYPE option<record<account>>;
DEFINE FIELD notes           ON payroll TYPE option<string>;
DEFINE FIELD created_at      ON payroll TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_payroll_emp_period ON payroll FIELDS employee, period_year, period_month UNIQUE;
DEFINE INDEX idx_payroll_status     ON payroll FIELDS status;

-- ============================================================
-- TABLE: attendance  (الحضور — Immutable)
-- ============================================================
DEFINE TABLE attendance SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD employee     ON attendance TYPE record<account>;
DEFINE FIELD event_type   ON attendance TYPE string
    ASSERT $value IN ["check_in", "check_out"];
DEFINE FIELD timestamp    ON attendance TYPE datetime VALUE $value OR time::now();
DEFINE FIELD source       ON attendance TYPE string
    ASSERT $value IN ["rfid", "web", "mobile", "manual"];
DEFINE FIELD metadata     ON attendance TYPE object  VALUE $value OR {};
DEFINE FIELD work_hours   ON attendance TYPE option<decimal>;

DEFINE INDEX idx_attendance_emp_time ON attendance FIELDS employee, timestamp;

-- ============================================================
-- TABLE: leave_request  (طلبات الإجازات)
-- ============================================================
DEFINE TABLE leave_request SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON leave_request TYPE bool VALUE $value OR false;
DEFINE FIELD employee    ON leave_request TYPE record<account>;
DEFINE FIELD leave_type  ON leave_request TYPE string
    ASSERT $value IN ["annual", "sick", "unpaid", "emergency", "maternity", "bereavement"];
DEFINE FIELD start_date  ON leave_request TYPE datetime;
DEFINE FIELD end_date    ON leave_request TYPE datetime;
DEFINE FIELD days        ON leave_request TYPE int   ASSERT $value > 0;
DEFINE FIELD reason      ON leave_request TYPE option<string>;
DEFINE FIELD status      ON leave_request TYPE string
    ASSERT $value IN ["pending", "approved", "rejected", "cancelled"]
    VALUE $value OR "pending";
DEFINE FIELD approved_by ON leave_request TYPE option<record<account>>;
DEFINE FIELD created_at  ON leave_request TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: leave_balance  (رصيد الإجازات)
-- ============================================================
DEFINE TABLE leave_balance SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee      ON leave_balance TYPE record<account>;
DEFINE FIELD year           ON leave_balance TYPE int;
DEFINE FIELD annual_total   ON leave_balance TYPE int   VALUE $value OR 21;
DEFINE FIELD annual_used    ON leave_balance TYPE int   VALUE $value OR 0;
DEFINE FIELD sick_total     ON leave_balance TYPE int   VALUE $value OR 10;
DEFINE FIELD sick_used      ON leave_balance TYPE int   VALUE $value OR 0;
DEFINE FIELD emergency_total ON leave_balance TYPE int  VALUE $value OR 5;
DEFINE FIELD emergency_used ON leave_balance TYPE int   VALUE $value OR 0;

DEFINE INDEX idx_leave_balance_emp_year ON leave_balance FIELDS employee, year UNIQUE;

-- ============================================================
-- TABLE: evaluation  (تقييم أداء الموظفين)
-- ============================================================
DEFINE TABLE evaluation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON evaluation TYPE bool VALUE $value OR false;
DEFINE FIELD employee       ON evaluation TYPE record<account>;
DEFINE FIELD evaluator      ON evaluation TYPE record<account>;
DEFINE FIELD period         ON evaluation TYPE string
    ASSERT $value IN ["q1", "q2", "q3", "q4", "annual"]
    VALUE $value OR "annual";
DEFINE FIELD year            ON evaluation TYPE int;
DEFINE FIELD productivity    ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD quality         ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD teamwork        ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD initiative      ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD attendance_score ON evaluation TYPE int  ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD overall_score   ON evaluation TYPE decimal VALUE $value OR ((productivity + quality + teamwork + initiative + attendance_score) / 5.0);
DEFINE FIELD strengths       ON evaluation TYPE option<string>;
DEFINE FIELD improvements    ON evaluation TYPE option<string>;
DEFINE FIELD goals           ON evaluation TYPE array  VALUE $value OR [];
DEFINE FIELD employee_notes  ON evaluation TYPE option<string>;
DEFINE FIELD status          ON evaluation TYPE string
    ASSERT $value IN ["draft", "submitted", "acknowledged"]
    VALUE $value OR "draft";
DEFINE FIELD created_at      ON evaluation TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_eval_emp_period ON evaluation FIELDS employee, year, period UNIQUE;

-- ============================================================
-- TABLE: trainee  (المتدربون)
-- ============================================================
DEFINE TABLE trainee SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON trainee TYPE bool VALUE $value OR false;
DEFINE FIELD name         ON trainee TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD phone        ON trainee TYPE string;
DEFINE FIELD email        ON trainee TYPE option<string>;
DEFINE FIELD institution  ON trainee TYPE string;
DEFINE FIELD start_date   ON trainee TYPE datetime;
DEFINE FIELD end_date     ON trainee TYPE datetime;
DEFINE FIELD supervisor   ON trainee TYPE option<record<account>>;
DEFINE FIELD status       ON trainee TYPE string
    ASSERT $value IN ["active", "completed", "dropped"]
    VALUE $value OR "active";
DEFINE FIELD notes        ON trainee TYPE option<string>;
DEFINE FIELD created_at   ON trainee TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: certificate  (شهادات — قابلة للتحقق العام عبر QR/URL)
-- ============================================================
DEFINE TABLE certificate SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD credential_id   ON certificate TYPE string;
DEFINE FIELD trainee         ON certificate TYPE option<record<trainee>>;
DEFINE FIELD trainee_name    ON certificate TYPE string;
DEFINE FIELD course_title    ON certificate TYPE string;
DEFINE FIELD total_hours     ON certificate TYPE int     ASSERT $value > 0;
DEFINE FIELD start_date      ON certificate TYPE datetime;
DEFINE FIELD end_date        ON certificate TYPE datetime;
DEFINE FIELD instructor      ON certificate TYPE string;
DEFINE FIELD issued_date     ON certificate TYPE datetime VALUE $value OR time::now();
DEFINE FIELD expiry_date     ON certificate TYPE option<datetime>;
DEFINE FIELD status          ON certificate TYPE string
    ASSERT $value IN ["issued", "revoked", "expired"]
    VALUE $value OR "issued";
DEFINE FIELD verification_hash ON certificate TYPE option<string>;
DEFINE FIELD pdf_url          ON certificate TYPE option<string>;
DEFINE FIELD qr_data          ON certificate TYPE option<string>;
DEFINE FIELD notes            ON certificate TYPE option<string>;

DEFINE INDEX idx_cert_credential ON certificate FIELDS credential_id UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٥: إدارة المشاريع (Project Management)           ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: project  (المشاريع — مملوكة لمنظمة أو للمنصة)
-- الشركات الناشئة والفرق التطوعية تدير مشاريعها من لوحة التحكم
-- ============================================================
DEFINE TABLE project SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id)
        FOR create WHERE $auth.role IN ["admin", "manager", "org_admin"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
            OR organization IN (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role IN ["owner", "admin"])
        FOR delete WHERE false;

DEFINE FIELD is_archived ON project TYPE bool VALUE $value OR false;
DEFINE FIELD organization ON project TYPE option<record<organization>>;
DEFINE FIELD title       ON project TYPE string;
DEFINE FIELD description ON project TYPE string;
DEFINE FIELD status      ON project TYPE string
    ASSERT $value IN ["planning", "active", "on_hold", "completed", "cancelled"]
    VALUE $value OR "planning";
DEFINE FIELD priority    ON project TYPE int ASSERT $value IN [1, 2, 3] VALUE $value OR 2;
DEFINE FIELD budget      ON project TYPE decimal   VALUE $value OR 0.0;
DEFINE FIELD currency    ON project TYPE string    VALUE $value OR "SYP";
DEFINE FIELD start_date  ON project TYPE datetime;
DEFINE FIELD deadline    ON project TYPE option<datetime>;
DEFINE FIELD metadata    ON project TYPE object VALUE $value OR {};
DEFINE FIELD created_at  ON project TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_project_org ON project FIELDS organization;

-- ============================================================
-- TABLE: task  (المهام)
-- ============================================================
DEFINE TABLE task SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
            OR project IN (SELECT VALUE id FROM project WHERE organization IN
                (SELECT VALUE organization FROM org_membership WHERE account = $auth.id))
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
            OR project IN (SELECT VALUE id FROM project WHERE organization IN
                (SELECT VALUE organization FROM org_membership WHERE account = $auth.id AND role IN ["owner", "admin"]))
        FOR delete WHERE false;

DEFINE FIELD is_archived ON task TYPE bool VALUE $value OR false;
DEFINE FIELD project     ON task TYPE record<project>;
DEFINE FIELD title       ON task TYPE string;
DEFINE FIELD description ON task TYPE option<string>;
DEFINE FIELD status      ON task TYPE string
    ASSERT $value IN ["todo", "in_progress", "review", "done"]
    VALUE $value OR "todo";
DEFINE FIELD priority    ON task TYPE int
    ASSERT $value IN [1, 2, 3]
    VALUE $value OR 2;
DEFINE FIELD due_date    ON task TYPE option<datetime>;
DEFINE FIELD estimated_hours ON task TYPE option<decimal>;
DEFINE FIELD actual_hours    ON task TYPE option<decimal>;
DEFINE FIELD created_at  ON task TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: timesheet  (تسجيل ساعات العمل على المشاريع)
-- ============================================================
DEFINE TABLE timesheet SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON timesheet TYPE bool VALUE $value OR false;
DEFINE FIELD employee      ON timesheet TYPE record<account>;
DEFINE FIELD project       ON timesheet TYPE record<project>;
DEFINE FIELD task           ON timesheet TYPE option<record<task>>;
DEFINE FIELD date           ON timesheet TYPE datetime;
DEFINE FIELD hours          ON timesheet TYPE decimal ASSERT $value > 0 AND $value <= 24;
DEFINE FIELD description    ON timesheet TYPE option<string>;
DEFINE FIELD billable       ON timesheet TYPE bool    VALUE $value OR true;
DEFINE FIELD hourly_rate    ON timesheet TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD status         ON timesheet TYPE string
    ASSERT $value IN ["draft", "submitted", "approved", "rejected"]
    VALUE $value OR "draft";
DEFINE FIELD approved_by    ON timesheet TYPE option<record<account>>;
DEFINE FIELD created_at     ON timesheet TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_timesheet_emp_date ON timesheet FIELDS employee, date;
DEFINE INDEX idx_timesheet_project  ON timesheet FIELDS project;

-- ============================================================
-- TABLE: report  (التقارير)
-- ============================================================
DEFINE TABLE report SCHEMAFULL
    PERMISSIONS
        FOR select WHERE author = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE author = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD is_archived ON report TYPE bool VALUE $value OR false;
DEFINE FIELD author      ON report TYPE record<account>;
DEFINE FIELD title       ON report TYPE string;
DEFINE FIELD content     ON report TYPE string;
DEFINE FIELD report_type ON report TYPE string
    ASSERT $value IN ["daily", "weekly", "incident", "technical", "financial"];
DEFINE FIELD attachments ON report TYPE array VALUE $value OR [];
DEFINE FIELD created_at  ON report TYPE datetime VALUE $value OR time::now();

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٦: المالية (Finance)                              ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: invoice  (الفواتير)
-- ============================================================
DEFINE TABLE invoice SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON invoice TYPE bool VALUE $value OR false;
DEFINE FIELD organization    ON invoice TYPE option<record<organization>>;
DEFINE FIELD invoice_number  ON invoice TYPE string;
DEFINE FIELD invoice_date    ON invoice TYPE datetime;
DEFINE FIELD due_date        ON invoice TYPE option<datetime>;
DEFINE FIELD items           ON invoice TYPE array VALUE $value OR [];
DEFINE FIELD subtotal        ON invoice TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD tax_rate        ON invoice TYPE decimal  VALUE $value OR 0.0;
DEFINE FIELD tax_amount      ON invoice TYPE decimal;
DEFINE FIELD total           ON invoice TYPE decimal;
DEFINE FIELD paid_amount     ON invoice TYPE decimal  VALUE $value OR 0.0;
DEFINE FIELD balance         ON invoice TYPE decimal;
DEFINE FIELD currency        ON invoice TYPE string   VALUE $value OR "SYP";
DEFINE FIELD status          ON invoice TYPE string
    ASSERT $value IN ["draft", "sent", "partially_paid", "paid", "overdue", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD notes           ON invoice TYPE option<string>;
DEFINE FIELD created_at      ON invoice TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_invoice_number ON invoice FIELDS invoice_number UNIQUE;
DEFINE INDEX idx_invoice_org    ON invoice FIELDS organization;

-- ============================================================
-- TABLE: payment  (المدفوعات — Immutable)
-- ============================================================
DEFINE TABLE payment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;

DEFINE FIELD invoice          ON payment TYPE record<invoice>;
DEFINE FIELD amount           ON payment TYPE decimal  ASSERT $value > 0;
DEFINE FIELD currency         ON payment TYPE string   VALUE $value OR "SYP";
DEFINE FIELD payment_date     ON payment TYPE datetime VALUE $value OR time::now();
DEFINE FIELD method           ON payment TYPE string
    ASSERT $value IN ["cash", "bank_transfer", "check", "online"];
DEFINE FIELD reference_number ON payment TYPE option<string>;
DEFINE FIELD received_by      ON payment TYPE option<record<account>>;
DEFINE FIELD notes            ON payment TYPE option<string>;
DEFINE FIELD created_at       ON payment TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_payment_invoice ON payment FIELDS invoice;

-- ============================================================
-- TABLE: expense  (المصروفات)
-- ============================================================
DEFINE TABLE expense SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON expense TYPE bool VALUE $value OR false;
DEFINE FIELD employee     ON expense TYPE record<account>;
DEFINE FIELD category     ON expense TYPE string
    ASSERT $value IN ["supplies", "utilities", "salary", "equipment", "travel", "maintenance", "rent", "communication", "events", "other"];
DEFINE FIELD amount       ON expense TYPE decimal  ASSERT $value > 0;
DEFINE FIELD currency     ON expense TYPE string   VALUE $value OR "SYP";
DEFINE FIELD date         ON expense TYPE datetime;
DEFINE FIELD description  ON expense TYPE string;
DEFINE FIELD receipt_url  ON expense TYPE option<string>;
DEFINE FIELD approved_by  ON expense TYPE option<record<account>>;
DEFINE FIELD status       ON expense TYPE string
    ASSERT $value IN ["pending", "approved", "rejected"]
    VALUE $value OR "pending";
DEFINE FIELD project      ON expense TYPE option<record<project>>;
DEFINE FIELD notes        ON expense TYPE option<string>;
DEFINE FIELD created_at   ON expense TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_expense_date ON expense FIELDS date;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٧: الأصول والمخزون (Assets & Inventory)          ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: asset  (الأصول — أجهزة، أدوات، أثاث)
-- ============================================================
DEFINE TABLE asset SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON asset TYPE bool VALUE $value OR false;
DEFINE FIELD name           ON asset TYPE string;
DEFINE FIELD category       ON asset TYPE string
    ASSERT $value IN ["electronics", "tools", "furniture", "vehicle", "software", "other"];
DEFINE FIELD serial_number  ON asset TYPE option<string>;
DEFINE FIELD purchase_date  ON asset TYPE option<datetime>;
DEFINE FIELD value          ON asset TYPE decimal     VALUE $value OR 0.0;
DEFINE FIELD location       ON asset TYPE option<string>;
DEFINE FIELD assigned_to    ON asset TYPE option<record<account>>;
DEFINE FIELD status         ON asset TYPE string
    ASSERT $value IN ["available", "in_use", "maintenance", "retired", "lost"]
    VALUE $value OR "available";
DEFINE FIELD notes          ON asset TYPE option<string>;
DEFINE FIELD created_at     ON asset TYPE datetime    VALUE $value OR time::now();

DEFINE INDEX idx_asset_serial ON asset FIELDS serial_number UNIQUE;

-- ============================================================
-- TABLE: inventory_item  (المخزون — مواد ومعدات)
-- ============================================================
DEFINE TABLE inventory_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON inventory_item TYPE bool VALUE $value OR false;
DEFINE FIELD sku           ON inventory_item TYPE string;
DEFINE FIELD name          ON inventory_item TYPE string;
DEFINE FIELD category      ON inventory_item TYPE string
    ASSERT $value IN ["office_supplies", "tech_equipment", "event_materials", "consumables", "other"];
DEFINE FIELD unit          ON inventory_item TYPE string VALUE $value OR "piece";
DEFINE FIELD quantity      ON inventory_item TYPE int     VALUE $value OR 0;
DEFINE FIELD min_quantity  ON inventory_item TYPE int     VALUE $value OR 5;
DEFINE FIELD unit_cost     ON inventory_item TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD currency      ON inventory_item TYPE string  VALUE $value OR "SYP";
DEFINE FIELD location      ON inventory_item TYPE option<string>;
DEFINE FIELD notes         ON inventory_item TYPE option<string>;
DEFINE FIELD created_at    ON inventory_item TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_inventory_sku ON inventory_item FIELDS sku UNIQUE;

-- ============================================================
-- TABLE: inventory_transaction  (حركات المخزون — Immutable)
-- ============================================================
DEFINE TABLE inventory_transaction SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;

DEFINE FIELD item        ON inventory_transaction TYPE record<inventory_item>;
DEFINE FIELD txn_type    ON inventory_transaction TYPE string
    ASSERT $value IN ["in", "out", "adjust", "return"];
DEFINE FIELD quantity    ON inventory_transaction TYPE int   ASSERT $value != 0;
DEFINE FIELD reference   ON inventory_transaction TYPE option<string>;
DEFINE FIELD employee    ON inventory_transaction TYPE option<record<account>>;
DEFINE FIELD notes       ON inventory_transaction TYPE option<string>;
DEFINE FIELD created_at  ON inventory_transaction TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_invtx_item ON inventory_transaction FIELDS item, created_at;

-- ============================================================
-- TABLE: warehouse  (المخازن / المواقع)
-- ============================================================
DEFINE TABLE warehouse SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD is_archived ON warehouse TYPE bool VALUE $value OR false;
DEFINE FIELD name          ON warehouse TYPE string  ASSERT string::len($value) > 0;
DEFINE FIELD code          ON warehouse TYPE string;
DEFINE FIELD address       ON warehouse TYPE option<string>;
DEFINE FIELD manager       ON warehouse TYPE option<record<account>>;
DEFINE FIELD is_active     ON warehouse TYPE bool    VALUE $value OR true;
DEFINE FIELD capacity      ON warehouse TYPE option<int>;
DEFINE FIELD notes         ON warehouse TYPE option<string>;
DEFINE FIELD created_at    ON warehouse TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_warehouse_code ON warehouse FIELDS code UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٨: الإشعارات والتدقيق (Notifications & Audit)    ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: notification  (الإشعارات)
-- ============================================================
DEFINE TABLE notification SCHEMAFULL
    PERMISSIONS
        FOR select WHERE recipient = $auth.id OR $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update WHERE recipient = $auth.id OR $auth.role = "admin"
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD recipient     ON notification TYPE record<account>;
DEFINE FIELD title         ON notification TYPE string;
DEFINE FIELD body          ON notification TYPE string;
DEFINE FIELD type          ON notification TYPE string
    ASSERT $value IN ["approval_required", "approval_result", "action_completed", "event_reminder", "alert", "info"]
    VALUE $value OR "info";
DEFINE FIELD related_table ON notification TYPE option<string>;
DEFINE FIELD related_id    ON notification TYPE option<string>;
DEFINE FIELD is_read       ON notification TYPE bool VALUE $value OR false;
DEFINE FIELD channel       ON notification TYPE string
    ASSERT $value IN ["dashboard", "email", "both"]
    VALUE $value OR "dashboard";
DEFINE FIELD created_at    ON notification TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_notif_recipient ON notification FIELDS recipient;
DEFINE INDEX idx_notif_unread    ON notification FIELDS recipient, is_read;

-- ============================================================
-- TABLE: audit_log  (سجل التدقيق — Immutable)
-- ============================================================
DEFINE TABLE audit_log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD actor       ON audit_log TYPE option<record<account>>;
DEFINE FIELD action      ON audit_log TYPE string
    ASSERT $value IN ["create", "update", "delete", "login", "logout", "export"];
DEFINE FIELD table_name  ON audit_log TYPE string;
DEFINE FIELD record_id   ON audit_log TYPE option<string>;
DEFINE FIELD old_data    ON audit_log TYPE option<object>;
DEFINE FIELD new_data    ON audit_log TYPE option<object>;
DEFINE FIELD ip_address  ON audit_log TYPE option<string>;
DEFINE FIELD user_agent  ON audit_log TYPE option<string>;
DEFINE FIELD timestamp   ON audit_log TYPE datetime VALUE $value OR time::now();
DEFINE FIELD prev_hash   ON audit_log TYPE option<string>;
DEFINE FIELD record_hash ON audit_log TYPE option<string>;

DEFINE INDEX idx_audit_time  ON audit_log FIELDS timestamp;
DEFINE INDEX idx_audit_actor ON audit_log FIELDS actor;
DEFINE INDEX idx_audit_hash  ON audit_log FIELDS record_hash UNIQUE;

-- ============================================================
-- TABLE: document_attachment  (المرفقات)
-- ============================================================
DEFINE TABLE document_attachment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE uploaded_by = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD is_archived ON document_attachment TYPE bool VALUE $value OR false;
DEFINE FIELD file_name       ON document_attachment TYPE string;
DEFINE FIELD file_path       ON document_attachment TYPE string;
DEFINE FIELD file_size       ON document_attachment TYPE int  ASSERT $value > 0;
DEFINE FIELD mime_type       ON document_attachment TYPE string;
DEFINE FIELD category        ON document_attachment TYPE string
    ASSERT $value IN ["contract", "invoice", "receipt", "manual", "photo", "report", "certificate", "id_document", "other"]
    VALUE $value OR "other";
DEFINE FIELD related_table   ON document_attachment TYPE string;
DEFINE FIELD related_id      ON document_attachment TYPE string;
DEFINE FIELD description     ON document_attachment TYPE option<string>;
DEFINE FIELD tags            ON document_attachment TYPE array  VALUE $value OR [];
DEFINE FIELD uploaded_by     ON document_attachment TYPE record<account>;
DEFINE FIELD is_public       ON document_attachment TYPE bool  VALUE $value OR false;
DEFINE FIELD version         ON document_attachment TYPE int   VALUE $value OR 1;
DEFINE FIELD created_at      ON document_attachment TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_doc_related ON document_attachment FIELDS related_table, related_id;

-- ============================================================
-- TABLE: log  (سجلات RFID الخام — للتدقيق الأمني)
-- ============================================================
DEFINE TABLE log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "security"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD rfid_uid    ON log TYPE string;
DEFINE FIELD device_id   ON log TYPE string;
DEFINE FIELD timestamp   ON log TYPE datetime VALUE $value OR time::now();
DEFINE FIELD action      ON log TYPE string;
DEFINE FIELD raw_payload ON log TYPE option<object>;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٩: البريد الإلكتروني (Email)                     ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: email_config  (إعدادات SMTP)
-- ============================================================
DEFINE TABLE email_config SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD host       ON email_config TYPE string;
DEFINE FIELD port       ON email_config TYPE int     ASSERT $value > 0 AND $value < 65536;
DEFINE FIELD username   ON email_config TYPE string;
DEFINE FIELD password   ON email_config TYPE string;
DEFINE FIELD from_email ON email_config TYPE string  ASSERT string::is::email($value);
DEFINE FIELD from_name  ON email_config TYPE string;
DEFINE FIELD use_tls    ON email_config TYPE bool    VALUE $value OR true;
DEFINE FIELD is_active  ON email_config TYPE bool    VALUE $value OR true;
DEFINE FIELD updated_at ON email_config TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: email_template  (قوالب البريد)
-- ============================================================
DEFINE TABLE email_template SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD name       ON email_template TYPE string;
DEFINE FIELD subject    ON email_template TYPE string;
DEFINE FIELD body       ON email_template TYPE string;
DEFINE FIELD category   ON email_template TYPE string
    ASSERT $value IN ["welcome", "event_invite", "payroll", "leave", "announcement", "custom"]
    VALUE $value OR "custom";
DEFINE FIELD variables  ON email_template TYPE array  VALUE $value OR [];
DEFINE FIELD is_active  ON email_template TYPE bool   VALUE $value OR true;
DEFINE FIELD created_at ON email_template TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_email_template_name ON email_template FIELDS name UNIQUE;

-- ============================================================
-- TABLE: email_log  (سجل الرسائل — Immutable)
-- ============================================================
DEFINE TABLE email_log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD recipient   ON email_log TYPE string;
DEFINE FIELD subject     ON email_log TYPE string;
DEFINE FIELD body        ON email_log TYPE option<string>;
DEFINE FIELD template    ON email_log TYPE option<record<email_template>>;
DEFINE FIELD status      ON email_log TYPE string
    ASSERT $value IN ["sent", "failed", "pending"]
    VALUE $value OR "pending";
DEFINE FIELD error_msg   ON email_log TYPE option<string>;
DEFINE FIELD sent_by     ON email_log TYPE option<record<account>>;
DEFINE FIELD sent_at     ON email_log TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_email_log_status ON email_log FIELDS status;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٠: وكلاء الذكاء الاصطناعي (AI Agents)           ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: agent_account  (هوية الوكيل + مفتاح API)
-- ============================================================
DEFINE TABLE agent_account SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD name         ON agent_account TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD description  ON agent_account TYPE option<string>;
DEFINE FIELD api_key_hash ON agent_account TYPE string;
DEFINE FIELD scopes       ON agent_account TYPE array    VALUE $value OR [];
DEFINE FIELD rate_limit   ON agent_account TYPE int      VALUE $value OR 100;
DEFINE FIELD is_active    ON agent_account TYPE bool     VALUE $value OR true;
DEFINE FIELD created_by   ON agent_account TYPE option<record<account>>;
DEFINE FIELD created_at   ON agent_account TYPE datetime VALUE $value OR time::now();
DEFINE FIELD last_used_at ON agent_account TYPE option<datetime>;

DEFINE INDEX idx_agent_name ON agent_account FIELDS name UNIQUE;

-- ============================================================
-- TABLE: action_policy  (سياسة تصنيف العمليات)
-- ============================================================
DEFINE TABLE action_policy SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD action           ON action_policy TYPE string;
DEFINE FIELD scope_required   ON action_policy TYPE string;
DEFINE FIELD sensitivity      ON action_policy TYPE string
    ASSERT $value IN ["routine", "sensitive", "critical"]
    VALUE $value OR "sensitive";
DEFINE FIELD auto_approve_conditions ON action_policy TYPE option<object>;
DEFINE FIELD requires_human   ON action_policy TYPE bool VALUE $value OR true;
DEFINE FIELD is_blocked       ON action_policy TYPE bool VALUE $value OR false;
DEFINE FIELD escalation_hours ON action_policy TYPE int  VALUE $value OR 24;
DEFINE FIELD description      ON action_policy TYPE option<string>;
DEFINE FIELD is_active        ON action_policy TYPE bool VALUE $value OR true;

DEFINE INDEX idx_action_policy_action ON action_policy FIELDS action UNIQUE;

-- ============================================================
-- TABLE: approval_request  (طلبات الموافقة)
-- ============================================================
DEFINE TABLE approval_request SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD agent         ON approval_request TYPE record<agent_account>;
DEFINE FIELD action        ON approval_request TYPE string;
DEFINE FIELD target_table  ON approval_request TYPE string;
DEFINE FIELD target_id     ON approval_request TYPE option<string>;
DEFINE FIELD payload       ON approval_request TYPE object;
DEFINE FIELD reason        ON approval_request TYPE option<string>;
DEFINE FIELD status        ON approval_request TYPE string
    ASSERT $value IN ["pending", "approved", "rejected", "expired", "executed"]
    VALUE $value OR "pending";
DEFINE FIELD priority      ON approval_request TYPE string
    ASSERT $value IN ["low", "normal", "high", "urgent"]
    VALUE $value OR "normal";
DEFINE FIELD reviewed_by   ON approval_request TYPE option<record<account>>;
DEFINE FIELD review_note   ON approval_request TYPE option<string>;
DEFINE FIELD created_at    ON approval_request TYPE datetime VALUE $value OR time::now();
DEFINE FIELD reviewed_at   ON approval_request TYPE option<datetime>;
DEFINE FIELD expires_at    ON approval_request TYPE option<datetime>;
DEFINE FIELD executed_at   ON approval_request TYPE option<datetime>;

DEFINE INDEX idx_approval_status  ON approval_request FIELDS status;
DEFINE INDEX idx_approval_agent   ON approval_request FIELDS agent;

-- ============================================================
-- TABLE: agent_usage_log  (سجل استخدام الوكلاء)
-- ============================================================
DEFINE TABLE agent_usage_log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD agent        ON agent_usage_log TYPE record<agent_account>;
DEFINE FIELD endpoint     ON agent_usage_log TYPE string;
DEFINE FIELD method       ON agent_usage_log TYPE string;
DEFINE FIELD action       ON agent_usage_log TYPE option<string>;
DEFINE FIELD status_code  ON agent_usage_log TYPE int;
DEFINE FIELD response_ms  ON agent_usage_log TYPE option<int>;
DEFINE FIELD was_approved ON agent_usage_log TYPE option<bool>;
DEFINE FIELD was_blocked  ON agent_usage_log TYPE option<bool>;
DEFINE FIELD ip_address   ON agent_usage_log TYPE option<string>;
DEFINE FIELD timestamp    ON agent_usage_log TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_agent_usage_agent ON agent_usage_log FIELDS agent;
DEFINE INDEX idx_agent_usage_time  ON agent_usage_log FIELDS timestamp;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١١: علاقات Graph                                  ██
-- ████████████████████████████████████████████████████████████████

-- account -> ASSIGNED_TO -> task
DEFINE TABLE assigned_to SCHEMAFULL TYPE RELATION IN account OUT task
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager", "org_admin"];
DEFINE FIELD assigned_at ON assigned_to TYPE datetime VALUE $value OR time::now();

-- account -> MEMBER_OF -> project
DEFINE TABLE member_of SCHEMAFULL TYPE RELATION IN account OUT project
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager", "org_admin"];
DEFINE FIELD role      ON member_of TYPE string
    ASSERT $value IN ["lead", "member", "observer"] VALUE $value OR "member";
DEFINE FIELD joined_at ON member_of TYPE datetime VALUE $value OR time::now();

-- account -> SUBMITTED -> report
DEFINE TABLE submitted SCHEMAFULL TYPE RELATION IN account OUT report
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";

-- account -> SUPERVISES -> trainee
DEFINE TABLE supervises SCHEMAFULL TYPE RELATION IN account OUT trainee
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD started_at ON supervises TYPE datetime VALUE $value OR time::now();

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٢: الأحداث التلقائية (Events)                   ██
-- ████████████████████████████████████████████████████████████████

-- عند التسجيل في فعالية: حدّث العداد
DEFINE EVENT on_event_registration ON event_registration
    WHEN $event = "CREATE"
    THEN {
        UPDATE $after.event SET registered_count += 1;
    };

-- عند إلغاء تسجيل: أنقص العداد
DEFINE EVENT on_event_cancel ON event_registration
    WHEN $before.status != "cancelled" AND $after.status = "cancelled"
    THEN {
        UPDATE $after.event SET registered_count -= 1;
    };

-- عند الموافقة على منظمة: أعطِ المؤسس دور org_admin
DEFINE EVENT on_org_approved ON organization
    WHEN $before.status != "approved" AND $after.status = "approved"
    THEN {
        LET $owner = (SELECT VALUE account FROM org_membership
            WHERE organization = $after.id AND role = "owner" LIMIT 1)[0];
        IF $owner != NONE THEN
            UPDATE $owner SET role = "org_admin";
        END;
    };

-- عند إنشاء دفعة: حدّث الفاتورة
DEFINE EVENT on_payment_created ON payment
    WHEN $event = "CREATE"
    THEN {
        LET $inv = (SELECT * FROM $after.invoice)[0];
        LET $new_paid = $inv.paid_amount + $after.amount;
        LET $new_balance = $inv.total - $new_paid;
        LET $new_status = IF $new_balance <= 0 THEN "paid"
                         ELSE IF $new_paid > 0 THEN "partially_paid"
                         ELSE $inv.status
                         END;
        UPDATE $after.invoice SET
            paid_amount = $new_paid,
            balance     = $new_balance,
            status      = $new_status;
    };

-- عند إخراج مخزون: قلّل الكمية
DEFINE EVENT on_inventory_out ON inventory_transaction
    WHEN $event = "CREATE" AND $after.txn_type = "out"
    THEN {
        UPDATE $after.item SET quantity -= $after.quantity;
    };

-- عند إدخال مخزون: زد الكمية
DEFINE EVENT on_inventory_in ON inventory_transaction
    WHEN $event = "CREATE" AND $after.txn_type = "in"
    THEN {
        UPDATE $after.item SET quantity += $after.quantity;
    };

-- عند إرجاع مخزون: زد الكمية
DEFINE EVENT on_inventory_return ON inventory_transaction
    WHEN $event = "CREATE" AND $after.txn_type = "return"
    THEN {
        UPDATE $after.item SET quantity += $after.quantity;
    };

-- عند الموافقة على إجازة: حدّث الرصيد
DEFINE EVENT on_leave_approved ON leave_request
    WHEN $before.status != "approved" AND $after.status = "approved"
    THEN {
        LET $yr = time::year($after.start_date);
        IF $after.leave_type = "annual" THEN
            UPDATE leave_balance SET annual_used += $after.days
                WHERE employee = $after.employee AND year = $yr;
        ELSE IF $after.leave_type = "sick" THEN
            UPDATE leave_balance SET sick_used += $after.days
                WHERE employee = $after.employee AND year = $yr;
        ELSE IF $after.leave_type = "emergency" THEN
            UPDATE leave_balance SET emergency_used += $after.days
                WHERE employee = $after.employee AND year = $yr;
        END;
    };

-- عند تسجيل خروج: احسب ساعات العمل
DEFINE EVENT on_checkout ON attendance
    WHEN $event = "CREATE" AND $after.event_type = "check_out"
    THEN {
        LET $last_checkin = (SELECT VALUE timestamp FROM attendance
            WHERE employee = $after.employee
              AND event_type = "check_in"
            ORDER BY timestamp DESC LIMIT 1)[0];
        IF $last_checkin != NONE THEN
            UPDATE $after.id SET work_hours = ($after.timestamp - $last_checkin) / 3600s;
        END;
    };

-- ████████████████████████████████████████████████████████████████
-- ██  أمثلة على الاستعلامات                                   ██
-- ████████████████████████████████████████████████████████████████

-- جميع الشركات الناشئة المعتمدة:
--   SELECT * FROM organization WHERE org_type = "startup" AND status = "approved";

-- جميع الفرق التطوعية:
--   SELECT * FROM organization WHERE org_type = "volunteer_team" AND status = "approved";

-- أعضاء منظمة معينة:
--   SELECT account.*, org_membership.role FROM org_membership
--       WHERE organization = organization:xyz FETCH account;

-- مشاريع منظمة:
--   SELECT * FROM project WHERE organization = organization:xyz;

-- الفعاليات القادمة:
--   SELECT * FROM event WHERE status = "published" AND start_date > time::now()
--       ORDER BY start_date ASC;

-- المسجلون في فعالية:
--   SELECT account.*, event_registration.status FROM event_registration
--       WHERE event = event:abc FETCH account;

-- جميع المنظمات التي يعمل بها حساب معين:
--   SELECT organization.* FROM org_membership
--       WHERE account = account:user1 FETCH organization;

-- مهام موظف معين:
--   SELECT ->assigned_to->task.* FROM account:ahmed;

-- أعضاء مشروع مع أدوارهم:
--   SELECT <-member_of<-account.*, member_of.role FROM project:proj1;
