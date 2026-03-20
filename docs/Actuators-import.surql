-- ==============================================================
-- Actuators ERP/CRM/HR System — SurrealDB Schema
-- ==============================================================
 -- النظام المتكامل لإدارة الموارد البشرية وعلاقات العملاء والعمليات.
-- ==============================================================

-- ============================================================
-- NAMESPACE & DATABASE
-- ============================================================
USE NS Actuators;
-- DEFINE DATABASE hr_system;
USE DB erb;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١: الهوية والصلاحيات (Identity & RBAC)           ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: account  (الهوية الموحدة - RBAC)
-- القاعدة: كل شخص له سجل account واحد بغض النظر عن طريقة دخوله.
-- auth_methods يدعم: RFID، بريد إلكتروني، معرّف الجهاز، وأي طريقة مستقبلية.
-- ============================================================
DEFINE TABLE account SCHEMAFULL
    PERMISSIONS
        FOR select WHERE id = $auth.id OR $auth.role = "admin"
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD email         ON account TYPE string   ASSERT string::is::email($value);
DEFINE FIELD password_hash ON account TYPE string;
DEFINE FIELD role          ON account TYPE string
    ASSERT $value IN ["admin", "manager", "employee", "intern", "applicant"]
    VALUE $value OR "applicant";
DEFINE FIELD is_active     ON account TYPE bool     VALUE $value OR true;
-- [{type: "rfid"|"email"|"device", value: "..."}]
DEFINE FIELD auth_methods  ON account TYPE array    VALUE $value OR [];
DEFINE FIELD created_at    ON account TYPE datetime VALUE $value OR time::now();
DEFINE FIELD last_login    ON account TYPE option<datetime>;

DEFINE INDEX idx_account_email ON account FIELDS email UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٢: الموارد البشرية (HR)                          ██
-- ████████████████████████████████████████████████████████████████

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

-- ============================================================
-- TABLE: employment_contract  (عقد العمل والشروط)
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
-- TABLE: salary  (الرواتب — مقيدة الوصول للـ Admin فقط)
-- لا يمكن لأي موظف الاطلاع على راتب غيره
-- ============================================================
DEFINE TABLE salary SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee       ON salary TYPE record<account>;
DEFINE FIELD base_amount    ON salary TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD currency       ON salary TYPE string   VALUE $value OR "SAR";
DEFINE FIELD effective_from ON salary TYPE datetime;
DEFINE FIELD effective_to   ON salary TYPE option<datetime>;
DEFINE FIELD notes          ON salary TYPE option<string>;

-- ============================================================
-- TABLE: payroll  (كشف الرواتب الشهري)
-- كل شهر = سطر لكل موظف يحتوي على الراتب الأساسي + البدلات - الخصومات
-- Immutable بعد الاعتماد — لا تعديل بعد status = "paid"
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
DEFINE FIELD gosi_employee   ON payroll TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD gosi_employer   ON payroll TYPE decimal VALUE $value OR 0.0;
-- حقل محسوب: صافي الراتب
DEFINE FIELD net_salary      ON payroll TYPE decimal VALUE $value OR (base_salary + housing_allowance + transport_allowance + other_allowances + overtime_amount - deductions - gosi_employee);
DEFINE FIELD currency        ON payroll TYPE string  VALUE $value OR "SAR";
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
-- TABLE: attendance  (الحضور — Immutable، لا تعديل أبداً)
-- كل حركة دخول/خروج = سطر جديد.
-- metadata يخزن: GPS لموظفي الريموت، RFID UID لموظفي الموقع.
-- ============================================================
DEFINE TABLE attendance SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;  -- ممنوع نهائياً

DEFINE FIELD employee     ON attendance TYPE record<account>;
DEFINE FIELD event_type   ON attendance TYPE string
    ASSERT $value IN ["check_in", "check_out"];
DEFINE FIELD timestamp    ON attendance TYPE datetime VALUE $value OR time::now();
DEFINE FIELD source       ON attendance TYPE string
    ASSERT $value IN ["rfid", "web", "mobile", "manual"];
-- {rfid_card, gps: {lat, lng}, device_id, ip}
DEFINE FIELD metadata     ON attendance TYPE object  VALUE $value OR {};
-- ساعات العمل — يُكتب بواسطة Event عند check_out (ليس محسوباً في وقت التشغيل)
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
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON leave_request TYPE bool VALUE  OR false;


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
-- TABLE: leave_balance  (رصيد الإجازات لكل موظف سنوياً)
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
-- تقييم دوري (ربع سنوي أو سنوي) لكل موظف
-- ============================================================
DEFINE TABLE evaluation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON evaluation TYPE bool VALUE  OR false;


DEFINE FIELD employee       ON evaluation TYPE record<account>;
DEFINE FIELD evaluator      ON evaluation TYPE record<account>;
DEFINE FIELD period         ON evaluation TYPE string
    ASSERT $value IN ["q1", "q2", "q3", "q4", "annual"]
    VALUE $value OR "annual";
DEFINE FIELD year            ON evaluation TYPE int;
-- مقاييس التقييم (1-5 لكل معيار)
DEFINE FIELD productivity    ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD quality         ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD teamwork        ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD initiative      ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD attendance_score ON evaluation TYPE int  ASSERT $value >= 1 AND $value <= 5;
-- المعدل العام (محسوب)
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
-- TABLE: log  (سجلات RFID الخام — للتدقيق الأمني فقط)
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

-- ============================================================
-- TABLE: trainee  (المتدربون)
-- ============================================================
DEFINE TABLE trainee SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON trainee TYPE bool VALUE  OR false;


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
-- TABLE: certificate  (شهادات التدريب — قابلة للتحقق العام عبر QR/URL)
-- ============================================================
DEFINE TABLE certificate SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true  -- قراءة عامة للتحقق من الشهادات
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
-- رابط التحقق العام: /verify/certificate/{credential_id}
DEFINE FIELD verification_hash ON certificate TYPE option<string>;
DEFINE FIELD pdf_url          ON certificate TYPE option<string>;
DEFINE FIELD qr_data          ON certificate TYPE option<string>;
DEFINE FIELD notes            ON certificate TYPE option<string>;

DEFINE INDEX idx_cert_credential ON certificate FIELDS credential_id UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٣: إدارة علاقات العملاء (CRM)                    ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: client  (العملاء)
-- ============================================================
DEFINE TABLE client SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON client TYPE bool VALUE  OR false;


DEFINE FIELD company_name    ON client TYPE string   ASSERT string::len($value) > 0;
DEFINE FIELD contact_person  ON client TYPE option<string>;
DEFINE FIELD email           ON client TYPE option<string>;
DEFINE FIELD phone           ON client TYPE option<string>;
DEFINE FIELD address         ON client TYPE option<string>;
DEFINE FIELD city            ON client TYPE option<string>;
DEFINE FIELD latitude        ON client TYPE option<decimal>;
DEFINE FIELD longitude       ON client TYPE option<decimal>;
DEFINE FIELD industry        ON client TYPE option<string>;
DEFINE FIELD credit_limit    ON client TYPE decimal   VALUE $value OR 0.0;
DEFINE FIELD status          ON client TYPE string
    ASSERT $value IN ["active", "inactive", "pending"]
    VALUE $value OR "pending";
DEFINE FIELD notes           ON client TYPE option<string>;
DEFINE FIELD created_at      ON client TYPE datetime  VALUE $value OR time::now();

DEFINE INDEX idx_client_email ON client FIELDS email UNIQUE;

-- ============================================================
-- TABLE: contact  (جهات الاتصال لكل عميل)
-- ============================================================
DEFINE TABLE contact SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON contact TYPE bool VALUE  OR false;


DEFINE FIELD client      ON contact TYPE record<client>;
DEFINE FIELD name        ON contact TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD title       ON contact TYPE option<string>;
DEFINE FIELD email       ON contact TYPE option<string>;
DEFINE FIELD phone       ON contact TYPE option<string>;
DEFINE FIELD is_primary  ON contact TYPE bool     VALUE $value OR false;
DEFINE FIELD notes       ON contact TYPE option<string>;
DEFINE FIELD created_at  ON contact TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_contact_client ON contact FIELDS client;

-- ============================================================
-- TABLE: interaction  (سجل التواصل مع العملاء)
-- قلب نظام الـ CRM — يسجل كل تفاعل مع العميل
-- ============================================================
DEFINE TABLE interaction SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE employee = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON interaction TYPE bool VALUE  OR false;


DEFINE FIELD client        ON interaction TYPE record<client>;
DEFINE FIELD contact       ON interaction TYPE option<record<contact>>;
DEFINE FIELD employee      ON interaction TYPE record<account>;
DEFINE FIELD channel       ON interaction TYPE string
    ASSERT $value IN ["call", "email", "meeting", "visit", "whatsapp", "other"];
DEFINE FIELD subject       ON interaction TYPE string;
DEFINE FIELD notes         ON interaction TYPE option<string>;
DEFINE FIELD scheduled_at  ON interaction TYPE option<datetime>;
DEFINE FIELD completed_at  ON interaction TYPE option<datetime>;
DEFINE FIELD follow_up     ON interaction TYPE option<datetime>;
DEFINE FIELD created_at    ON interaction TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_interaction_client ON interaction FIELDS client, created_at;

-- ============================================================
-- TABLE: opportunity  (الفرص التجارية — Sales Pipeline)
-- ============================================================
DEFINE TABLE opportunity SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON opportunity TYPE bool VALUE  OR false;


DEFINE FIELD client         ON opportunity TYPE record<client>;
DEFINE FIELD title          ON opportunity TYPE string;
DEFINE FIELD description    ON opportunity TYPE option<string>;
DEFINE FIELD value          ON opportunity TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD currency       ON opportunity TYPE string   VALUE $value OR "SAR";
DEFINE FIELD stage          ON opportunity TYPE string
    ASSERT $value IN ["lead", "qualified", "proposal", "negotiation", "won", "lost"]
    VALUE $value OR "lead";
DEFINE FIELD probability    ON opportunity TYPE int
    ASSERT $value >= 0 AND $value <= 100
    VALUE $value OR 10;
DEFINE FIELD expected_close ON opportunity TYPE option<datetime>;
DEFINE FIELD assigned_to    ON opportunity TYPE option<record<account>>;
DEFINE FIELD lost_reason    ON opportunity TYPE option<string>;
DEFINE FIELD notes          ON opportunity TYPE option<string>;
DEFINE FIELD created_at     ON opportunity TYPE datetime VALUE $value OR time::now();
DEFINE FIELD updated_at     ON opportunity TYPE datetime VALUE time::now();

DEFINE INDEX idx_opportunity_client ON opportunity FIELDS client;
DEFINE INDEX idx_opportunity_stage  ON opportunity FIELDS stage;

-- ============================================================
-- TABLE: quotation  (عروض الأسعار)
-- ============================================================
DEFINE TABLE quotation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON quotation TYPE bool VALUE  OR false;


DEFINE FIELD client         ON quotation TYPE record<client>;
DEFINE FIELD opportunity    ON quotation TYPE option<record<opportunity>>;
DEFINE FIELD quote_number   ON quotation TYPE string;
-- [{description, quantity, unit_price, total}]
DEFINE FIELD items          ON quotation TYPE array VALUE $value OR [];
DEFINE FIELD subtotal       ON quotation TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD tax_rate       ON quotation TYPE decimal  VALUE $value OR 15.0;
DEFINE FIELD tax_amount     ON quotation TYPE decimal;
DEFINE FIELD total          ON quotation TYPE decimal;
DEFINE FIELD validity_date  ON quotation TYPE option<datetime>;
DEFINE FIELD status         ON quotation TYPE string
    ASSERT $value IN ["draft", "sent", "accepted", "rejected", "expired"]
    VALUE $value OR "draft";
DEFINE FIELD notes          ON quotation TYPE option<string>;
DEFINE FIELD created_at     ON quotation TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_quotation_number ON quotation FIELDS quote_number UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٤: المالية (Finance)                              ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: invoice  (الفواتير)
-- ============================================================
DEFINE TABLE invoice SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON invoice TYPE bool VALUE  OR false;


DEFINE FIELD client          ON invoice TYPE record<client>;
DEFINE FIELD invoice_number  ON invoice TYPE string;
DEFINE FIELD invoice_date    ON invoice TYPE datetime;
DEFINE FIELD due_date        ON invoice TYPE option<datetime>;
-- [{description, quantity, unit_price, total}]
DEFINE FIELD items           ON invoice TYPE array VALUE $value OR [];
DEFINE FIELD subtotal        ON invoice TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD tax_rate        ON invoice TYPE decimal  VALUE $value OR 15.0;
DEFINE FIELD tax_amount      ON invoice TYPE decimal;
DEFINE FIELD total           ON invoice TYPE decimal;
DEFINE FIELD paid_amount     ON invoice TYPE decimal  VALUE $value OR 0.0;
DEFINE FIELD balance         ON invoice TYPE decimal;
DEFINE FIELD currency        ON invoice TYPE string   VALUE $value OR "SAR";
DEFINE FIELD status          ON invoice TYPE string
    ASSERT $value IN ["draft", "sent", "partially_paid", "paid", "overdue", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD notes           ON invoice TYPE option<string>;
DEFINE FIELD created_at      ON invoice TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_invoice_number ON invoice FIELDS invoice_number UNIQUE;
DEFINE INDEX idx_invoice_client ON invoice FIELDS client;

-- ============================================================
-- TABLE: payment  (المدفوعات — Immutable)
-- كل عملية دفع = سطر جديد. لا تعديل ولا حذف.
-- ============================================================
DEFINE TABLE payment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;  -- Immutable

DEFINE FIELD invoice          ON payment TYPE record<invoice>;
DEFINE FIELD amount           ON payment TYPE decimal  ASSERT $value > 0;
DEFINE FIELD currency         ON payment TYPE string   VALUE $value OR "SAR";
DEFINE FIELD payment_date     ON payment TYPE datetime VALUE $value OR time::now();
DEFINE FIELD method           ON payment TYPE string
    ASSERT $value IN ["cash", "bank_transfer", "check", "credit_card", "online"];
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
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON expense TYPE bool VALUE  OR false;


DEFINE FIELD employee     ON expense TYPE record<account>;
DEFINE FIELD category     ON expense TYPE string
    ASSERT $value IN ["supplies", "utilities", "salary", "equipment", "travel", "maintenance", "rent", "communication", "other"];
DEFINE FIELD amount       ON expense TYPE decimal  ASSERT $value > 0;
DEFINE FIELD currency     ON expense TYPE string   VALUE $value OR "SAR";
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

-- ============================================================
-- TABLE: tax_config  (إعدادات الضرائب)
-- لإدارة ضريبة القيمة المضافة والاستقطاع
-- ============================================================
DEFINE TABLE tax_config SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD name           ON tax_config TYPE string;
DEFINE FIELD code           ON tax_config TYPE string;
DEFINE FIELD rate           ON tax_config TYPE decimal  ASSERT $value >= 0 AND $value <= 100;
DEFINE FIELD tax_type       ON tax_config TYPE string
    ASSERT $value IN ["vat", "withholding", "excise", "customs", "other"]
    VALUE $value OR "vat";
DEFINE FIELD is_default      ON tax_config TYPE bool  VALUE $value OR false;
DEFINE FIELD is_active       ON tax_config TYPE bool  VALUE $value OR true;
DEFINE FIELD applies_to      ON tax_config TYPE string
    ASSERT $value IN ["sales", "purchases", "both"]
    VALUE $value OR "both";
DEFINE FIELD notes           ON tax_config TYPE option<string>;
DEFINE FIELD created_at      ON tax_config TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_tax_code ON tax_config FIELDS code UNIQUE;

-- ============================================================
-- TABLE: cost_center  (مراكز التكلفة)
-- لتحميل المصاريف على أقسام أو مشاريع
-- ============================================================
DEFINE TABLE cost_center SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD name         ON cost_center TYPE string;
DEFINE FIELD code         ON cost_center TYPE string;
DEFINE FIELD parent       ON cost_center TYPE option<record<cost_center>>;
DEFINE FIELD manager      ON cost_center TYPE option<record<account>>;
DEFINE FIELD budget        ON cost_center TYPE decimal  VALUE $value OR 0.0;
DEFINE FIELD currency      ON cost_center TYPE string   VALUE $value OR "SAR";
DEFINE FIELD is_active     ON cost_center TYPE bool  VALUE $value OR true;
DEFINE FIELD notes         ON cost_center TYPE option<string>;
DEFINE FIELD created_at    ON cost_center TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_cc_code ON cost_center FIELDS code UNIQUE;

-- ============================================================
-- TABLE: currency  (العملات وأسعار الصرف)
-- ============================================================
DEFINE TABLE currency SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD code            ON currency TYPE string  ASSERT string::len($value) = 3;
DEFINE FIELD name            ON currency TYPE string;
DEFINE FIELD name_ar         ON currency TYPE option<string>;
DEFINE FIELD symbol          ON currency TYPE string;
DEFINE FIELD exchange_rate   ON currency TYPE decimal  ASSERT $value > 0;
DEFINE FIELD base_currency   ON currency TYPE bool  VALUE $value OR false;
DEFINE FIELD is_active       ON currency TYPE bool  VALUE $value OR true;
DEFINE FIELD last_updated    ON currency TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_currency_code ON currency FIELDS code UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٤.٢: المشتريات وسلاسل الإمداد (Procurement)      ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: vendor  (الموردون)
-- ============================================================
DEFINE TABLE vendor SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON vendor TYPE bool VALUE  OR false;


DEFINE FIELD company_name    ON vendor TYPE string;
DEFINE FIELD contact_person  ON vendor TYPE option<string>;
DEFINE FIELD email           ON vendor TYPE option<string>;
DEFINE FIELD phone           ON vendor TYPE option<string>;
DEFINE FIELD address         ON vendor TYPE option<string>;
DEFINE FIELD city            ON vendor TYPE option<string>;
DEFINE FIELD country         ON vendor TYPE string  VALUE $value OR "SA";
DEFINE FIELD tax_number      ON vendor TYPE option<string>;
DEFINE FIELD payment_terms   ON vendor TYPE string
    ASSERT $value IN ["cash", "net_15", "net_30", "net_60", "net_90"]
    VALUE $value OR "net_30";
DEFINE FIELD currency        ON vendor TYPE string  VALUE $value OR "SAR";
DEFINE FIELD bank_name       ON vendor TYPE option<string>;
DEFINE FIELD bank_iban       ON vendor TYPE option<string>;
DEFINE FIELD rating          ON vendor TYPE option<int>;
DEFINE FIELD categories      ON vendor TYPE array  VALUE $value OR [];
DEFINE FIELD status          ON vendor TYPE string
    ASSERT $value IN ["active", "inactive", "blacklisted"]
    VALUE $value OR "active";
DEFINE FIELD notes           ON vendor TYPE option<string>;
DEFINE FIELD created_at      ON vendor TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_vendor_name ON vendor FIELDS company_name;
DEFINE INDEX idx_vendor_tax  ON vendor FIELDS tax_number UNIQUE;

-- ============================================================
-- TABLE: purchase_order  (أمر شراء)
-- يوثق طلب الشراء قبل التحول لفاتورة
-- ============================================================
DEFINE TABLE purchase_order SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON purchase_order TYPE bool VALUE  OR false;


DEFINE FIELD po_number       ON purchase_order TYPE string;
DEFINE FIELD vendor          ON purchase_order TYPE record<vendor>;
DEFINE FIELD requested_by    ON purchase_order TYPE record<account>;
DEFINE FIELD approved_by     ON purchase_order TYPE option<record<account>>;
DEFINE FIELD order_date      ON purchase_order TYPE datetime VALUE $value OR time::now();
DEFINE FIELD expected_date   ON purchase_order TYPE option<datetime>;
-- [{item: record<inventory_item>, description, quantity, unit_price, total}]
DEFINE FIELD items           ON purchase_order TYPE array  VALUE $value OR [];
DEFINE FIELD subtotal        ON purchase_order TYPE decimal ASSERT $value >= 0;
DEFINE FIELD tax_rate        ON purchase_order TYPE decimal VALUE $value OR 15.0;
DEFINE FIELD tax_amount      ON purchase_order TYPE decimal;
DEFINE FIELD total           ON purchase_order TYPE decimal;
DEFINE FIELD currency        ON purchase_order TYPE string  VALUE $value OR "SAR";
DEFINE FIELD cost_center     ON purchase_order TYPE option<record<cost_center>>;
DEFINE FIELD warehouse       ON purchase_order TYPE option<record<warehouse>>;
DEFINE FIELD status          ON purchase_order TYPE string
    ASSERT $value IN ["draft", "pending_approval", "approved", "ordered", "partially_received", "received", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD notes           ON purchase_order TYPE option<string>;
DEFINE FIELD created_at      ON purchase_order TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_po_number ON purchase_order FIELDS po_number UNIQUE;
DEFINE INDEX idx_po_vendor ON purchase_order FIELDS vendor;
DEFINE INDEX idx_po_status ON purchase_order FIELDS status;

-- ============================================================
-- TABLE: purchase_receipt  (إيصال استلام)
-- لمطابقة ما تم طلبه مع ما تم استلامه فعلياً
-- ============================================================
DEFINE TABLE purchase_receipt SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON purchase_receipt TYPE bool VALUE  OR false;


DEFINE FIELD purchase_order   ON purchase_receipt TYPE record<purchase_order>;
DEFINE FIELD receipt_number   ON purchase_receipt TYPE string;
DEFINE FIELD received_by     ON purchase_receipt TYPE record<account>;
DEFINE FIELD receipt_date    ON purchase_receipt TYPE datetime VALUE $value OR time::now();
DEFINE FIELD warehouse       ON purchase_receipt TYPE record<warehouse>;
-- [{item: record<inventory_item>, ordered_qty, received_qty, accepted_qty, rejected_qty, rejection_reason}]
DEFINE FIELD items           ON purchase_receipt TYPE array  VALUE $value OR [];
DEFINE FIELD quality_check   ON purchase_receipt TYPE bool   VALUE $value OR false;
DEFINE FIELD status          ON purchase_receipt TYPE string
    ASSERT $value IN ["draft", "inspecting", "accepted", "partial_accept", "rejected"]
    VALUE $value OR "draft";
DEFINE FIELD notes           ON purchase_receipt TYPE option<string>;
DEFINE FIELD created_at      ON purchase_receipt TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_receipt_number ON purchase_receipt FIELDS receipt_number UNIQUE;
DEFINE INDEX idx_receipt_po     ON purchase_receipt FIELDS purchase_order;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٥: الأصول والمخزون (Assets & Inventory)          ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: asset  (الأصول الداخلية — أجهزة، أدوات، أثاث)
-- ============================================================
DEFINE TABLE asset SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON asset TYPE bool VALUE  OR false;


DEFINE FIELD name           ON asset TYPE string;
DEFINE FIELD category       ON asset TYPE string
    ASSERT $value IN ["electronics", "tools", "furniture", "vehicle", "software", "other"];
DEFINE FIELD serial_number  ON asset TYPE option<string>;
DEFINE FIELD purchase_date  ON asset TYPE option<datetime>;
DEFINE FIELD value          ON asset TYPE decimal     VALUE $value OR 0.0;
DEFINE FIELD depreciation   ON asset TYPE decimal     VALUE $value OR 0.0;
DEFINE FIELD location       ON asset TYPE option<string>;
DEFINE FIELD assigned_to    ON asset TYPE option<record<account>>;
DEFINE FIELD status         ON asset TYPE string
    ASSERT $value IN ["available", "in_use", "maintenance", "retired", "lost"]
    VALUE $value OR "available";
DEFINE FIELD notes          ON asset TYPE option<string>;
DEFINE FIELD created_at     ON asset TYPE datetime    VALUE $value OR time::now();

DEFINE INDEX idx_asset_serial ON asset FIELDS serial_number UNIQUE;

-- ============================================================
-- TABLE: machine  (ماكينات العملاء — للصيانة)
-- ============================================================
DEFINE TABLE machine SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON machine TYPE bool VALUE  OR false;


DEFINE FIELD client          ON machine TYPE record<client>;
DEFINE FIELD serial_number   ON machine TYPE string;
DEFINE FIELD model           ON machine TYPE string;
DEFINE FIELD manufacturer    ON machine TYPE option<string>;
DEFINE FIELD purchase_date   ON machine TYPE option<datetime>;
DEFINE FIELD warranty_end    ON machine TYPE option<datetime>;
DEFINE FIELD status          ON machine TYPE string
    ASSERT $value IN ["working", "needs_repair", "under_repair", "out_of_service"]
    VALUE $value OR "working";
DEFINE FIELD location        ON machine TYPE option<string>;
DEFINE FIELD notes           ON machine TYPE option<string>;
DEFINE FIELD created_at      ON machine TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_machine_serial ON machine FIELDS serial_number UNIQUE;
DEFINE INDEX idx_machine_client ON machine FIELDS client;

-- ============================================================
-- TABLE: inventory_item  (المخزون — قطع غيار ومواد)
-- ============================================================
DEFINE TABLE inventory_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON inventory_item TYPE bool VALUE  OR false;


DEFINE FIELD sku           ON inventory_item TYPE string;
DEFINE FIELD name          ON inventory_item TYPE string;
DEFINE FIELD category      ON inventory_item TYPE string
    ASSERT $value IN ["spare_parts", "raw_materials", "consumables", "packaging", "other"];
DEFINE FIELD unit          ON inventory_item TYPE string
    VALUE $value OR "piece";
DEFINE FIELD quantity      ON inventory_item TYPE int     VALUE $value OR 0;
DEFINE FIELD min_quantity  ON inventory_item TYPE int     VALUE $value OR 5;
DEFINE FIELD unit_cost     ON inventory_item TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD currency      ON inventory_item TYPE string  VALUE $value OR "SAR";
DEFINE FIELD location      ON inventory_item TYPE option<string>;
DEFINE FIELD supplier      ON inventory_item TYPE option<string>;
DEFINE FIELD notes         ON inventory_item TYPE option<string>;
DEFINE FIELD created_at    ON inventory_item TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_inventory_sku ON inventory_item FIELDS sku UNIQUE;

-- ============================================================
-- TABLE: inventory_transaction  (حركات المخزون — Immutable)
-- كل حركة إدخال/إخراج/تعديل = سطر جديد
-- ============================================================
DEFINE TABLE inventory_transaction SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;  -- Immutable

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
-- كل مخزن = موقع فعلي لتخزين الأصناف
-- ============================================================
DEFINE TABLE warehouse SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON warehouse TYPE bool VALUE  OR false;


DEFINE FIELD name          ON warehouse TYPE string  ASSERT string::len($value) > 0;
DEFINE FIELD code          ON warehouse TYPE string;
DEFINE FIELD address       ON warehouse TYPE option<string>;
DEFINE FIELD manager       ON warehouse TYPE option<record<account>>;
DEFINE FIELD is_active     ON warehouse TYPE bool    VALUE $value OR true;
DEFINE FIELD capacity      ON warehouse TYPE option<int>;
DEFINE FIELD notes         ON warehouse TYPE option<string>;
DEFINE FIELD created_at    ON warehouse TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_warehouse_code ON warehouse FIELDS code UNIQUE;

-- ============================================================
-- TABLE: stock_movement  (حركة البضائع بين المخازن — Immutable)
-- كل نقل بين مخزنين = سطر جديد لا يُعدّل
-- ============================================================
DEFINE TABLE stock_movement SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;  -- Immutable

DEFINE FIELD item            ON stock_movement TYPE record<inventory_item>;
DEFINE FIELD from_warehouse  ON stock_movement TYPE record<warehouse>;
DEFINE FIELD to_warehouse    ON stock_movement TYPE record<warehouse>;
DEFINE FIELD quantity        ON stock_movement TYPE int   ASSERT $value > 0;
DEFINE FIELD employee        ON stock_movement TYPE record<account>;
DEFINE FIELD reason          ON stock_movement TYPE option<string>;
DEFINE FIELD status          ON stock_movement TYPE string
    ASSERT $value IN ["pending", "in_transit", "received", "cancelled"]
    VALUE $value OR "pending";
DEFINE FIELD shipped_at      ON stock_movement TYPE option<datetime>;
DEFINE FIELD received_at     ON stock_movement TYPE option<datetime>;
DEFINE FIELD created_at      ON stock_movement TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_stock_mv_item ON stock_movement FIELDS item;
DEFINE INDEX idx_stock_mv_from ON stock_movement FIELDS from_warehouse;
DEFINE INDEX idx_stock_mv_to   ON stock_movement FIELDS to_warehouse;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٦: الصيانة والخدمات (Repairs & Services)         ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: repair_operation  (عمليات الصيانة والإصلاح)
-- ============================================================
DEFINE TABLE repair_operation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON repair_operation TYPE bool VALUE  OR false;


DEFINE FIELD machine        ON repair_operation TYPE record<machine>;
DEFINE FIELD project        ON repair_operation TYPE option<record<project>>;
DEFINE FIELD employee       ON repair_operation TYPE record<account>;
DEFINE FIELD description    ON repair_operation TYPE string;
DEFINE FIELD diagnosis      ON repair_operation TYPE option<string>;
-- [{item: record<inventory_item>, quantity, cost}]
DEFINE FIELD parts_used     ON repair_operation TYPE array  VALUE $value OR [];
DEFINE FIELD labor_hours    ON repair_operation TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD cost           ON repair_operation TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD warranty_claim ON repair_operation TYPE bool    VALUE $value OR false;
DEFINE FIELD start_time     ON repair_operation TYPE datetime VALUE $value OR time::now();
DEFINE FIELD end_time       ON repair_operation TYPE option<datetime>;
DEFINE FIELD status         ON repair_operation TYPE string
    ASSERT $value IN ["new", "diagnosing", "waiting_parts", "in_progress", "testing", "completed", "cancelled"]
    VALUE $value OR "new";
DEFINE FIELD notes          ON repair_operation TYPE option<string>;

DEFINE INDEX idx_repair_machine ON repair_operation FIELDS machine;
DEFINE INDEX idx_repair_status  ON repair_operation FIELDS status;

-- ============================================================
-- TABLE: service_contract  (عقود الصيانة الدورية)
-- ============================================================
DEFINE TABLE service_contract SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON service_contract TYPE bool VALUE  OR false;


DEFINE FIELD client         ON service_contract TYPE record<client>;
-- [record<machine>]
DEFINE FIELD machines       ON service_contract TYPE array VALUE $value OR [];
DEFINE FIELD title          ON service_contract TYPE string;
DEFINE FIELD start_date     ON service_contract TYPE datetime;
DEFINE FIELD end_date       ON service_contract TYPE datetime;
DEFINE FIELD value          ON service_contract TYPE decimal  ASSERT $value >= 0;
DEFINE FIELD currency       ON service_contract TYPE string   VALUE $value OR "SAR";
DEFINE FIELD frequency      ON service_contract TYPE string
    ASSERT $value IN ["monthly", "quarterly", "semi_annual", "annual"];
DEFINE FIELD next_service   ON service_contract TYPE option<datetime>;
DEFINE FIELD status         ON service_contract TYPE string
    ASSERT $value IN ["active", "expired", "suspended", "cancelled"]
    VALUE $value OR "active";
DEFINE FIELD notes          ON service_contract TYPE option<string>;
DEFINE FIELD created_at     ON service_contract TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_svc_contract_client ON service_contract FIELDS client;

-- ============================================================
-- TABLE: warranty_card  (بطاقات الضمان — قابلة للتحقق العام عبر QR/URL)
-- /verify/warranty/{warranty_number}
-- ============================================================
DEFINE TABLE warranty_card SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true  -- قراءة عامة للتحقق من الضمان
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete

DEFINE FIELD is_archived ON warranty_card TYPE bool VALUE  OR false;

DEFINE FIELD warranty_number  ON warranty_card TYPE string;
DEFINE FIELD client           ON warranty_card TYPE option<record<client>>;
DEFINE FIELD client_name      ON warranty_card TYPE option<string>;
DEFINE FIELD machine          ON warranty_card TYPE option<record<machine>>;
DEFINE FIELD product_name     ON warranty_card TYPE string;
DEFINE FIELD serial_number    ON warranty_card TYPE option<string>;
DEFINE FIELD model            ON warranty_card TYPE option<string>;
DEFINE FIELD purchase_date    ON warranty_card TYPE datetime;
DEFINE FIELD warranty_start   ON warranty_card TYPE datetime;
DEFINE FIELD warranty_end     ON warranty_card TYPE datetime;
DEFINE FIELD warranty_months  ON warranty_card TYPE int    ASSERT $value > 0;
DEFINE FIELD coverage_type    ON warranty_card TYPE string
    ASSERT $value IN ["full", "parts_only", "labor_only", "limited"]
    VALUE $value OR "full";
DEFINE FIELD terms            ON warranty_card TYPE option<string>;
DEFINE FIELD exclusions       ON warranty_card TYPE option<string>;
DEFINE FIELD max_claims       ON warranty_card TYPE option<int>;
DEFINE FIELD claims_used      ON warranty_card TYPE int  VALUE $value OR 0;
DEFINE FIELD status           ON warranty_card TYPE string
    ASSERT $value IN ["active", "expired", "voided", "claimed"]
    VALUE $value OR "active";
-- رابط التحقق العام: /verify/warranty/{warranty_number}
DEFINE FIELD verification_hash ON warranty_card TYPE option<string>;
DEFINE FIELD qr_data           ON warranty_card TYPE option<string>;
DEFINE FIELD issued_by         ON warranty_card TYPE option<record<account>>;
DEFINE FIELD notes             ON warranty_card TYPE option<string>;
DEFINE FIELD created_at        ON warranty_card TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_warranty_number ON warranty_card FIELDS warranty_number UNIQUE;
DEFINE INDEX idx_warranty_client ON warranty_card FIELDS client;
DEFINE INDEX idx_warranty_serial ON warranty_card FIELDS serial_number;
DEFINE INDEX idx_warranty_status ON warranty_card FIELDS status;

-- ============================================================
-- TABLE: maintenance_schedule  (جدول الصيانة الوقائية)
-- مواعيد دورية للصيانة (كل X أشهر أو كل Y ساعة عمل)
-- ============================================================
DEFINE TABLE maintenance_schedule SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON maintenance_schedule TYPE bool VALUE  OR false;


DEFINE FIELD machine          ON maintenance_schedule TYPE record<machine>;
DEFINE FIELD title            ON maintenance_schedule TYPE string;
DEFINE FIELD description      ON maintenance_schedule TYPE option<string>;
DEFINE FIELD frequency_type   ON maintenance_schedule TYPE string
    ASSERT $value IN ["calendar", "hours", "cycles"]
    VALUE $value OR "calendar";
-- عدد الأيام/الساعات/الدورات بين كل صيانة
DEFINE FIELD frequency_value  ON maintenance_schedule TYPE int   ASSERT $value > 0;
DEFINE FIELD last_performed   ON maintenance_schedule TYPE option<datetime>;
DEFINE FIELD next_due         ON maintenance_schedule TYPE option<datetime>;
DEFINE FIELD assigned_to      ON maintenance_schedule TYPE option<record<account>>;
-- [{item: record<inventory_item>, quantity}]
DEFINE FIELD required_parts   ON maintenance_schedule TYPE array  VALUE $value OR [];
DEFINE FIELD estimated_hours  ON maintenance_schedule TYPE decimal VALUE $value OR 1.0;
DEFINE FIELD priority         ON maintenance_schedule TYPE string
    ASSERT $value IN ["low", "medium", "high", "critical"]
    VALUE $value OR "medium";
DEFINE FIELD is_active        ON maintenance_schedule TYPE bool  VALUE $value OR true;
DEFINE FIELD notes            ON maintenance_schedule TYPE option<string>;
DEFINE FIELD created_at       ON maintenance_schedule TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_maint_machine ON maintenance_schedule FIELDS machine;
DEFINE INDEX idx_maint_next    ON maintenance_schedule FIELDS next_due;

-- ============================================================
-- TABLE: spare_part  (قطع الغيار — ربط المخزون بالآلات)
-- يربط أصناف المخزون بالآلات التي تحتاجها
-- ============================================================
DEFINE TABLE spare_part SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON spare_part TYPE bool VALUE  OR false;


DEFINE FIELD machine          ON spare_part TYPE record<machine>;
DEFINE FIELD inventory_item   ON spare_part TYPE record<inventory_item>;
DEFINE FIELD compatibility    ON spare_part TYPE string
    ASSERT $value IN ["exact_fit", "compatible", "alternative"]
    VALUE $value OR "exact_fit";
DEFINE FIELD quantity_needed  ON spare_part TYPE int  VALUE $value OR 1;
DEFINE FIELD is_critical      ON spare_part TYPE bool VALUE $value OR false;
DEFINE FIELD notes            ON spare_part TYPE option<string>;
DEFINE FIELD created_at       ON spare_part TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_spare_machine ON spare_part FIELDS machine;
DEFINE INDEX idx_spare_item    ON spare_part FIELDS inventory_item;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٦.٢: التصنيع والإنتاج (Manufacturing / MRP)      ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: bom  (قائمة المواد - Bill of Materials)
-- "الوصفة" لإنتاج منتج أو صيانة آلة
-- ============================================================
DEFINE TABLE bom SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON bom TYPE bool VALUE  OR false;


DEFINE FIELD name             ON bom TYPE string;
DEFINE FIELD code             ON bom TYPE string;
DEFINE FIELD bom_type         ON bom TYPE string
    ASSERT $value IN ["production", "maintenance", "assembly"]
    VALUE $value OR "production";
DEFINE FIELD output_item      ON bom TYPE option<record<inventory_item>>;
DEFINE FIELD machine          ON bom TYPE option<record<machine>>;
DEFINE FIELD version          ON bom TYPE int  VALUE $value OR 1;
-- [{item: record<inventory_item>, quantity, unit, notes}]
DEFINE FIELD components       ON bom TYPE array  VALUE $value OR [];
DEFINE FIELD labor_hours      ON bom TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD cost_estimate    ON bom TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD is_active        ON bom TYPE bool  VALUE $value OR true;
DEFINE FIELD notes            ON bom TYPE option<string>;
DEFINE FIELD created_at       ON bom TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_bom_code ON bom FIELDS code UNIQUE;

-- ============================================================
-- TABLE: production_line  (خطوط الإنتاج)
-- ربط الآلات بمواقعها التشغيلية
-- ============================================================
DEFINE TABLE production_line SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON production_line TYPE bool VALUE  OR false;


DEFINE FIELD name             ON production_line TYPE string;
DEFINE FIELD code             ON production_line TYPE string;
DEFINE FIELD warehouse        ON production_line TYPE option<record<warehouse>>;
-- [record<machine>]
DEFINE FIELD machines         ON production_line TYPE array  VALUE $value OR [];
DEFINE FIELD supervisor       ON production_line TYPE option<record<account>>;
DEFINE FIELD capacity_per_hour ON production_line TYPE option<decimal>;
DEFINE FIELD status           ON production_line TYPE string
    ASSERT $value IN ["active", "idle", "maintenance", "decommissioned"]
    VALUE $value OR "active";
DEFINE FIELD notes            ON production_line TYPE option<string>;
DEFINE FIELD created_at       ON production_line TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_prodline_code ON production_line FIELDS code UNIQUE;

-- ============================================================
-- TABLE: work_order  (أمر عمل)
-- تتبع عملية الإنتاج أو الإصلاح خطوة بخطوة
-- ============================================================
DEFINE TABLE work_order SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON work_order TYPE bool VALUE  OR false;


DEFINE FIELD wo_number        ON work_order TYPE string;
DEFINE FIELD wo_type          ON work_order TYPE string
    ASSERT $value IN ["production", "maintenance", "repair", "assembly"]
    VALUE $value OR "production";
DEFINE FIELD bom              ON work_order TYPE option<record<bom>>;
DEFINE FIELD machine          ON work_order TYPE option<record<machine>>;
DEFINE FIELD production_line  ON work_order TYPE option<record<production_line>>;
DEFINE FIELD repair_operation ON work_order TYPE option<record<repair_operation>>;
DEFINE FIELD maintenance_schedule ON work_order TYPE option<record<maintenance_schedule>>;
DEFINE FIELD assigned_to      ON work_order TYPE option<record<account>>;
DEFINE FIELD requested_by     ON work_order TYPE record<account>;
DEFINE FIELD planned_start    ON work_order TYPE option<datetime>;
DEFINE FIELD planned_end      ON work_order TYPE option<datetime>;
DEFINE FIELD actual_start     ON work_order TYPE option<datetime>;
DEFINE FIELD actual_end       ON work_order TYPE option<datetime>;
DEFINE FIELD quantity_planned ON work_order TYPE int  VALUE $value OR 1;
DEFINE FIELD quantity_produced ON work_order TYPE int VALUE $value OR 0;
DEFINE FIELD quantity_rejected ON work_order TYPE int VALUE $value OR 0;
-- [{item: record<inventory_item>, planned_qty, actual_qty}]
DEFINE FIELD materials_used   ON work_order TYPE array  VALUE $value OR [];
DEFINE FIELD labor_hours      ON work_order TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD cost             ON work_order TYPE decimal VALUE $value OR 0.0;
DEFINE FIELD priority         ON work_order TYPE string
    ASSERT $value IN ["low", "medium", "high", "urgent"]
    VALUE $value OR "medium";
DEFINE FIELD status           ON work_order TYPE string
    ASSERT $value IN ["draft", "planned", "in_progress", "paused", "completed", "cancelled"]
    VALUE $value OR "draft";
DEFINE FIELD notes            ON work_order TYPE option<string>;
DEFINE FIELD created_at       ON work_order TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_wo_number ON work_order FIELDS wo_number UNIQUE;
DEFINE INDEX idx_wo_status ON work_order FIELDS status;
DEFINE INDEX idx_wo_machine ON work_order FIELDS machine;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٧: إدارة المشاريع (Project Management)           ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: project
-- ============================================================
DEFINE TABLE project SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON project TYPE bool VALUE  OR false;


DEFINE FIELD client      ON project TYPE option<record<client>>;
DEFINE FIELD title       ON project TYPE string;
DEFINE FIELD description ON project TYPE string;
DEFINE FIELD status      ON project TYPE string
    ASSERT $value IN ["planning", "active", "on_hold", "completed", "cancelled"]
    VALUE $value OR "planning";
DEFINE FIELD priority    ON project TYPE int ASSERT $value IN [1, 2, 3] VALUE $value OR 2;
DEFINE FIELD budget      ON project TYPE decimal   VALUE $value OR 0.0;
DEFINE FIELD currency    ON project TYPE string    VALUE $value OR "SAR";
DEFINE FIELD start_date  ON project TYPE datetime;
DEFINE FIELD deadline    ON project TYPE option<datetime>;
DEFINE FIELD metadata    ON project TYPE object VALUE $value OR {};
DEFINE FIELD created_at  ON project TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: task
-- ============================================================
DEFINE TABLE task SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON task TYPE bool VALUE  OR false;


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
-- TABLE: timesheet  (تسجيل ساعات العمل على المشاريع والمهام)
-- يربط ساعات عمل الموظف بتكلفة المشروع
-- ============================================================
DEFINE TABLE timesheet SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON timesheet TYPE bool VALUE  OR false;


DEFINE FIELD employee      ON timesheet TYPE record<account>;
DEFINE FIELD project       ON timesheet TYPE record<project>;
DEFINE FIELD task           ON timesheet TYPE option<record<task>>;
DEFINE FIELD date           ON timesheet TYPE datetime;
DEFINE FIELD hours          ON timesheet TYPE decimal ASSERT $value > 0 AND $value <= 24;
DEFINE FIELD description    ON timesheet TYPE option<string>;
DEFINE FIELD billable       ON timesheet TYPE bool    VALUE $value OR true;
DEFINE FIELD hourly_rate    ON timesheet TYPE decimal VALUE $value OR 0.0;
-- حقل محسوب: التكلفة = الساعات * سعر الساعة
DEFINE FIELD cost           ON timesheet TYPE decimal
    VALUE $value.hours * $value.hourly_rate;
DEFINE FIELD status         ON timesheet TYPE string
    ASSERT $value IN ["draft", "submitted", "approved", "rejected"]
    VALUE $value OR "draft";
DEFINE FIELD approved_by    ON timesheet TYPE option<record<account>>;
DEFINE FIELD created_at     ON timesheet TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_timesheet_emp_date ON timesheet FIELDS employee, date;
DEFINE INDEX idx_timesheet_project  ON timesheet FIELDS project;

-- ============================================================
-- TABLE: report
-- ============================================================
DEFINE TABLE report SCHEMAFULL
    PERMISSIONS
        FOR select WHERE author = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE author = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON report TYPE bool VALUE  OR false;


DEFINE FIELD author      ON report TYPE record<account>;
DEFINE FIELD title       ON report TYPE string;
DEFINE FIELD content     ON report TYPE string;
DEFINE FIELD report_type ON report TYPE string
    ASSERT $value IN ["daily", "weekly", "incident", "technical", "financial"];
DEFINE FIELD attachments ON report TYPE array VALUE $value OR [];
DEFINE FIELD created_at  ON report TYPE datetime VALUE $value OR time::now();

-- ============================================================
-- TABLE: job_application  (التوظيف)
-- ============================================================
DEFINE TABLE job_application SCHEMAFULL
    PERMISSIONS
        FOR select WHERE applicant = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE true
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON job_application TYPE bool VALUE  OR false;


DEFINE FIELD applicant   ON job_application TYPE record<account>;
DEFINE FIELD position    ON job_application TYPE string;
DEFINE FIELD status      ON job_application TYPE string
    ASSERT $value IN ["pending", "reviewing", "interviewed", "accepted", "rejected"]
    VALUE $value OR "pending";
DEFINE FIELD applied_at  ON job_application TYPE datetime VALUE $value OR time::now();
DEFINE FIELD notes       ON job_application TYPE option<string>;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٨: الإشعارات والتدقيق (Notifications & Audit)    ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: notification
-- ============================================================
DEFINE TABLE notification SCHEMAFULL
    PERMISSIONS
        FOR select WHERE recipient = $auth.id OR $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update WHERE recipient = $auth.id  -- لتعليم كمقروء
        FOR delete WHERE recipient = $auth.id OR $auth.role = "admin";

DEFINE FIELD recipient   ON notification TYPE record<account>;
DEFINE FIELD title       ON notification TYPE string;
DEFINE FIELD message     ON notification TYPE string;
DEFINE FIELD notif_type  ON notification TYPE string
    ASSERT $value IN ["info", "warning", "success", "error", "reminder"];
DEFINE FIELD is_read     ON notification TYPE bool     VALUE $value OR false;
DEFINE FIELD link        ON notification TYPE option<string>;
DEFINE FIELD created_at  ON notification TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_notif_recipient ON notification FIELDS recipient, is_read;

-- ============================================================
-- TABLE: audit_log  (سجل التدقيق — Immutable مطلقاً)
-- يسجل كل عملية تعديل أو حذف على النظام
-- ============================================================
DEFINE TABLE audit_log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;  -- ممنوع نهائياً

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
-- Micro-Blockchain: سلسلة تشفيرية لضمان عدم التلاعب
-- prev_hash = هاش السجل السابق | record_hash = SHA-256(prev_hash + actor + action + table_name + record_id + timestamp)
DEFINE FIELD prev_hash   ON audit_log TYPE option<string>;
DEFINE FIELD record_hash ON audit_log TYPE option<string>;

DEFINE INDEX idx_audit_time  ON audit_log FIELDS timestamp;
DEFINE INDEX idx_audit_actor ON audit_log FIELDS actor;
DEFINE INDEX idx_audit_hash  ON audit_log FIELDS record_hash UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٩: علاقات Graph                                  ██
-- ████████████████████████████████████████████████████████████████

-- account -> ASSIGNED_TO -> task
DEFINE TABLE assigned_to SCHEMAFULL TYPE RELATION IN account OUT task
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD assigned_at ON assigned_to TYPE datetime VALUE $value OR time::now();

-- account -> MEMBER_OF -> project
DEFINE TABLE member_of SCHEMAFULL TYPE RELATION IN account OUT project
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD role      ON member_of TYPE string
    ASSERT $value IN ["lead", "member", "observer"] VALUE $value OR "member";
DEFINE FIELD joined_at ON member_of TYPE datetime VALUE $value OR time::now();

-- account -> SUBMITTED -> report
DEFINE TABLE submitted SCHEMAFULL TYPE RELATION IN account OUT report
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";

-- account -> CONTACTED -> client  (من تواصل مع العميل)
DEFINE TABLE contacted SCHEMAFULL TYPE RELATION IN account OUT client
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";
DEFINE FIELD channel      ON contacted TYPE string;
DEFINE FIELD contacted_at ON contacted TYPE datetime VALUE $value OR time::now();

-- account -> SUPERVISES -> trainee  (مشرف المتدرب)
DEFINE TABLE supervises SCHEMAFULL TYPE RELATION IN account OUT trainee
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD started_at ON supervises TYPE datetime VALUE $value OR time::now();

-- machine -> OWNED_BY -> client  (ملكية الماكينة)
DEFINE TABLE owned_by SCHEMAFULL TYPE RELATION IN machine OUT client
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD since ON owned_by TYPE datetime VALUE $value OR time::now();

-- inventory_item -> USED_IN -> repair_operation  (قطع غيار مستخدمة)
DEFINE TABLE used_in SCHEMAFULL TYPE RELATION IN inventory_item OUT repair_operation
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";
DEFINE FIELD quantity ON used_in TYPE int   ASSERT $value > 0;
DEFINE FIELD cost     ON used_in TYPE decimal VALUE $value OR 0.0;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٠: الأحداث التلقائية (Events)                   ██
-- ████████████████████████████████████████████████████████████████

-- عند قبول طلب توظيف: أنشئ عقد عمل تلقائياً وغيّر دور الحساب
DEFINE EVENT on_applicant_accepted ON job_application
    WHEN $before.status != "accepted" AND $after.status = "accepted"
    THEN {
        CREATE employment_contract SET
            employee        = $after.applicant,
            position        = $after.position,
            department      = "pending_assignment",
            employment_type = "full_time",
            start_date      = time::now(),
            work_location   = "onsite",
            is_active       = true;
        UPDATE $after.applicant SET role = "employee";
    };

-- عند تسجيل RFID check_in: أنشئ سجل حضور تلقائياً
DEFINE EVENT on_rfid_checkin ON log
    WHEN $event = "CREATE" AND $after.action = "check_in"
    THEN {
        CREATE attendance SET
            employee   = (SELECT VALUE id FROM account
                          WHERE auth_methods[WHERE type = "rfid"].value
                          CONTAINS $after.rfid_uid)[0],
            event_type = "check_in",
            source     = "rfid",
            metadata   = { rfid_uid: $after.rfid_uid, device_id: $after.device_id };
    };

-- عند فوز فرصة تجارية: أنشئ مشروع تلقائياً
DEFINE EVENT on_opportunity_won ON opportunity
    WHEN $before.stage != "won" AND $after.stage = "won"
    THEN {
        CREATE project SET
            client      = $after.client,
            title       = $after.title,
            description = $after.description OR "",
            status      = "planning",
            budget      = $after.value,
            currency    = $after.currency,
            start_date  = time::now();
    };

-- عند قبول عرض أسعار: أنشئ فاتورة تلقائياً
DEFINE EVENT on_quotation_accepted ON quotation
    WHEN $before.status != "accepted" AND $after.status = "accepted"
    THEN {
        CREATE invoice SET
            client         = $after.client,
            invoice_number = "INV-" + time::format(time::now(), "%Y") + "-AUTO",
            invoice_date   = time::now(),
            items          = $after.items,
            subtotal       = $after.subtotal,
            tax_rate       = $after.tax_rate,
            tax_amount     = $after.tax_amount,
            total          = $after.total,
            paid_amount    = 0.0,
            balance        = $after.total,
            status         = "sent";
    };

-- عند إنشاء دفعة: حدّث الفاتورة تلقائياً
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

-- عند إخراج مخزون: قلّل الكمية تلقائياً
DEFINE EVENT on_inventory_out ON inventory_transaction
    WHEN $event = "CREATE" AND $after.txn_type = "out"
    THEN {
        UPDATE $after.item SET quantity -= $after.quantity;
    };

-- عند إدخال مخزون: زد الكمية تلقائياً
DEFINE EVENT on_inventory_in ON inventory_transaction
    WHEN $event = "CREATE" AND $after.txn_type = "in"
    THEN {
        UPDATE $after.item SET quantity += $after.quantity;
    };

-- عند إرجاع مخزون: زد الكمية تلقائياً
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

-- عند اكتمال عملية صيانة: حدّث حالة الماكينة
DEFINE EVENT on_repair_completed ON repair_operation
    WHEN $before.status != "completed" AND $after.status = "completed"
    THEN {
        UPDATE $after.machine SET status = "working";
    };

-- عند استلام أمر شراء: أنشئ حركة مخزون تلقائياً
DEFINE EVENT on_receipt_accepted ON purchase_receipt
    WHEN $event = "CREATE" AND $after.status = "accepted"
    THEN {
        FOR $line IN $after.items {
            CREATE inventory_transaction SET
                item = $line.item,
                txn_type = "in",
                quantity = $line.accepted_qty,
                reference = "PO-Receipt: " + <string>$after.receipt_number,
                employee = $after.received_by;
        };
    };

-- عند حلول موعد صيانة وقائية: أنشئ أمر عمل تلقائياً
DEFINE EVENT on_maintenance_due ON maintenance_schedule
    WHEN $after.is_active = true AND $after.next_due != NONE AND $after.next_due <= time::now()
    THEN {
        CREATE work_order SET
            wo_number = "WO-PM-" + <string>time::unix(time::now()),
            wo_type = "maintenance",
            machine = $after.machine,
            maintenance_schedule = $after.id,
            assigned_to = $after.assigned_to,
            requested_by = $after.assigned_to,
            priority = $after.priority,
            materials_used = $after.required_parts,
            status = "planned";
    };

-- عند اكتمال أمر عمل إنتاج: أضف المنتج للمخزون
DEFINE EVENT on_work_order_completed ON work_order
    WHEN $before.status != "completed" AND $after.status = "completed" AND $after.wo_type = "production"
    THEN {
        LET $bom_data = (SELECT output_item FROM bom WHERE id = $after.bom)[0];
        IF $bom_data.output_item != NONE THEN
            CREATE inventory_transaction SET
                item = $bom_data.output_item,
                txn_type = "in",
                quantity = $after.quantity_produced,
                reference = "WO: " + <string>$after.wo_number,
                employee = $after.assigned_to;
        END;
    };

-- عند تسجيل خروج: احسب ساعات العمل واكتبها في السجل (بدلاً من الحساب في وقت الاستعلام)
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

-- عند إنشاء سجل تدقيق: احسب الهاش لبناء سلسلة الكتل المصغرة
-- يُنفَّذ عبر Rust/Application Layer لأن SurrealQL لا يوفر crypto::sha256 مباشرة
-- ملاحظة: في Rust يجب حساب prev_hash + record_hash قبل INSERT
-- DEFINE EVENT on_audit_created ON audit_log
--     WHEN $event = "CREATE"
--     THEN {
--         LET $prev = (SELECT VALUE record_hash FROM audit_log
--             ORDER BY timestamp DESC LIMIT 1)[0] OR "GENESIS";
--         UPDATE $after.id SET
--             prev_hash = $prev,
--             record_hash = crypto::sha256($prev + <string>$after.actor + $after.action + $after.table_name + <string>$after.record_id + <string>$after.timestamp);
--     };

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٠.٥: الحذف المنطقي (Soft Delete — is_archived)  ██
-- ████████████████████████████████████████████████████████████████
-- جميع الجداول الرئيسية تستخدم FOR delete WHERE false
-- بدلاً من الحذف الفعلي، اضبط is_archived = true
-- الاستعلامات يجب أن تتضمن WHERE is_archived = false

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١١: إدارة المستندات والأمان (DMS & Security)     ██
-- ████████████████████████████████████████████████████████████████

-- ============================================================
-- TABLE: document_attachment  (المرفقات والمستندات)
-- ربط ملفات بأي جدول (عقود، صور أعطال، كتيبات)
-- ============================================================
DEFINE TABLE document_attachment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE uploaded_by = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;  -- Soft Delete: استخدم is_archived بدلاً من الحذف

DEFINE FIELD is_archived ON document_attachment TYPE bool VALUE  OR false;


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
DEFINE INDEX idx_doc_category ON document_attachment FIELDS category;

-- ============================================================
-- TABLE: role_permission  (الأدوار والصلاحيات التفصيلية)
-- صلاحيات دقيقة لكل دور على كل جدول وعملية
-- ============================================================
DEFINE TABLE role_permission SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role = "admin"
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD role            ON role_permission TYPE string;
DEFINE FIELD resource        ON role_permission TYPE string;
DEFINE FIELD can_create      ON role_permission TYPE bool  VALUE $value OR false;
DEFINE FIELD can_read        ON role_permission TYPE bool  VALUE $value OR false;
DEFINE FIELD can_update      ON role_permission TYPE bool  VALUE $value OR false;
DEFINE FIELD can_delete      ON role_permission TYPE bool  VALUE $value OR false;
DEFINE FIELD can_export      ON role_permission TYPE bool  VALUE $value OR false;
DEFINE FIELD field_restrictions ON role_permission TYPE array VALUE $value OR [];
DEFINE FIELD conditions      ON role_permission TYPE option<string>;
DEFINE FIELD is_active       ON role_permission TYPE bool  VALUE $value OR true;
DEFINE FIELD created_at      ON role_permission TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_role_perm ON role_permission FIELDS role, resource UNIQUE;

-- ============================================================
-- أمثلة على Graph Queries
-- ============================================================
-- جميع مهام موظف معين:
--   SELECT ->assigned_to->task.* FROM account:ahmed;
--
-- أعضاء مشروع مع أدوارهم:
--   SELECT <-member_of<-account.*, member_of.role FROM project:drone_fw;
--
-- موظفون مع مهامهم غير المنتهية:
--   SELECT name, ->member_of->project->(->>task WHERE status != "done") AS pending
--   FROM account WHERE role = "employee";
--
-- جميع التفاعلات مع عميل:
--   SELECT <-contacted<-account.*, contacted.channel FROM client:acme;
--
-- ماكينات العميل مع حالتها:
--   SELECT <-owned_by<-machine.* FROM client:acme;
--
-- قطع الغيار المستخدمة في صيانة:
--   SELECT <-used_in<-inventory_item.*, used_in.quantity FROM repair_operation:r001;
--
-- رصيد إجازات موظف:
--   SELECT * FROM leave_balance WHERE employee = account:ahmed AND year = 2026;
--
-- Sales Pipeline — فرص مفتوحة مرتبة حسب القيمة:
--   SELECT * FROM opportunity WHERE stage NOT IN ["won", "lost"]
--       ORDER BY value DESC LIMIT 20;
--
-- إجمالي الإيرادات الشهرية:
--   SELECT math::sum(total) AS revenue FROM invoice
--       WHERE status IN ["paid", "partially_paid"]
--       AND invoice_date > time::now() - 30d;
--
-- جميع أوامر الشراء من مورد معين:
--   SELECT * FROM purchase_order WHERE vendor = vendor:abc
--       AND status NOT IN ["cancelled"] ORDER BY order_date DESC;
--
-- أوامر العمل المعلقة لآلة:
--   SELECT * FROM work_order WHERE machine = machine:m01
--       AND status IN ["planned", "in_progress"] ORDER BY priority DESC;
--
-- جدول الصيانة القادمة:
--   SELECT *, machine.model AS machine_name FROM maintenance_schedule
--       WHERE is_active = true AND next_due <= time::now() + 7d
--       ORDER BY next_due ASC;
--
-- مستندات مرتبطة بماكينة:
--   SELECT * FROM document_attachment
--       WHERE related_table = "machine" AND related_id = "machine:m01";
--
-- صلاحيات دور معين:
--   SELECT * FROM role_permission WHERE role = "manager" AND is_active = true;


-- ============================================================
-- TABLE: email_config  (إعدادات خادم SMTP — قابلة للتعديل من لوحة التحكم)
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
-- TABLE: email_template  (قوالب البريد الإلكتروني)
-- ============================================================
DEFINE TABLE email_template SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD name       ON email_template TYPE string;
DEFINE FIELD subject    ON email_template TYPE string;
DEFINE FIELD body       ON email_template TYPE string;
DEFINE FIELD category   ON email_template TYPE string
    ASSERT $value IN ["welcome", "payroll", "leave", "announcement", "custom"]
    VALUE $value OR "custom";
DEFINE FIELD variables  ON email_template TYPE array  VALUE $value OR [];
DEFINE FIELD is_active  ON email_template TYPE bool   VALUE $value OR true;
DEFINE FIELD created_at ON email_template TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_email_template_name ON email_template FIELDS name UNIQUE;

-- ============================================================
-- TABLE: email_log  (سجل الرسائل المرسلة — Immutable)
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
DEFINE INDEX idx_email_log_date   ON email_log FIELDS sent_at;

-- ############################################################
-- AI AGENT INFRASTRUCTURE — البنية التحتية لوكلاء الذكاء الاصطناعي
-- ############################################################

-- ============================================================
-- TABLE: agent_account  (هوية الوكيل + مفتاح API + الصلاحيات)
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
-- TABLE: approval_request  (طلبات الموافقة البشرية)
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
DEFINE INDEX idx_approval_created ON approval_request FIELDS created_at;

-- ============================================================
-- TABLE: notification  (نظام الإشعارات)
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
    ASSERT $value IN ["approval_required", "approval_result", "action_completed", "alert", "info"]
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

-- ############################################################
-- E-COMMERCE STORE — المتجر الإلكتروني
-- ############################################################

DEFINE TABLE store_settings SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD store_name     ON store_settings TYPE string;
DEFINE FIELD store_name_en  ON store_settings TYPE option<string>;
DEFINE FIELD description    ON store_settings TYPE option<string>;
DEFINE FIELD logo_url       ON store_settings TYPE option<string>;
DEFINE FIELD currency       ON store_settings TYPE string  VALUE $value OR "SAR";
DEFINE FIELD tax_rate       ON store_settings TYPE option<float>;
DEFINE FIELD shipping_flat  ON store_settings TYPE option<float>;
DEFINE FIELD free_shipping_above ON store_settings TYPE option<float>;
DEFINE FIELD is_active      ON store_settings TYPE bool VALUE $value OR false;
DEFINE FIELD contact_email  ON store_settings TYPE option<string>;
DEFINE FIELD contact_phone  ON store_settings TYPE option<string>;
DEFINE FIELD updated_at     ON store_settings TYPE datetime VALUE $value OR time::now();

DEFINE TABLE store_order SCHEMAFULL
    PERMISSIONS
        FOR select WHERE customer = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD order_number   ON store_order TYPE string;
DEFINE FIELD customer       ON store_order TYPE option<record<client>>;
DEFINE FIELD customer_name  ON store_order TYPE string;
DEFINE FIELD customer_email ON store_order TYPE option<string>;
DEFINE FIELD customer_phone ON store_order TYPE option<string>;
DEFINE FIELD status         ON store_order TYPE string
    ASSERT $value IN ["pending", "confirmed", "processing", "shipped", "delivered", "cancelled", "refunded"]
    VALUE $value OR "pending";
DEFINE FIELD subtotal       ON store_order TYPE float VALUE $value OR 0;
DEFINE FIELD tax_amount     ON store_order TYPE float VALUE $value OR 0;
DEFINE FIELD shipping_cost  ON store_order TYPE float VALUE $value OR 0;
DEFINE FIELD discount       ON store_order TYPE float VALUE $value OR 0;
DEFINE FIELD total          ON store_order TYPE float VALUE $value OR 0;
DEFINE FIELD currency       ON store_order TYPE string VALUE $value OR "SAR";
DEFINE FIELD shipping_address ON store_order TYPE option<object>;
DEFINE FIELD notes          ON store_order TYPE option<string>;
DEFINE FIELD created_at     ON store_order TYPE datetime VALUE $value OR time::now();
DEFINE FIELD updated_at     ON store_order TYPE option<datetime>;

DEFINE INDEX idx_order_number   ON store_order FIELDS order_number UNIQUE;
DEFINE INDEX idx_order_customer ON store_order FIELDS customer;
DEFINE INDEX idx_order_status   ON store_order FIELDS status;
DEFINE INDEX idx_order_date     ON store_order FIELDS created_at;

DEFINE TABLE order_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD order          ON order_item TYPE record<store_order>;
DEFINE FIELD item_type      ON order_item TYPE string
    ASSERT $value IN ["product", "service"]
    VALUE $value OR "product";
DEFINE FIELD product        ON order_item TYPE option<record<product>>;
DEFINE FIELD service        ON order_item TYPE option<record<service_catalog>>;
DEFINE FIELD name           ON order_item TYPE string;
DEFINE FIELD sku            ON order_item TYPE option<string>;
DEFINE FIELD quantity       ON order_item TYPE int VALUE $value OR 1;
DEFINE FIELD unit_price     ON order_item TYPE float;
DEFINE FIELD total_price    ON order_item TYPE float;
DEFINE FIELD notes          ON order_item TYPE option<string>;

DEFINE INDEX idx_order_item_order ON order_item FIELDS order;

DEFINE TABLE shopping_cart SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE owner = $auth.id OR $auth.role = "admin";

DEFINE FIELD owner          ON shopping_cart TYPE option<record<client>>;
DEFINE FIELD session_id     ON shopping_cart TYPE option<string>;
DEFINE FIELD item_type      ON shopping_cart TYPE string
    ASSERT $value IN ["product", "service"];
DEFINE FIELD product        ON shopping_cart TYPE option<record<product>>;
DEFINE FIELD service        ON shopping_cart TYPE option<record<service_catalog>>;
DEFINE FIELD name           ON shopping_cart TYPE string;
DEFINE FIELD quantity       ON shopping_cart TYPE int VALUE $value OR 1;
DEFINE FIELD unit_price     ON shopping_cart TYPE float;
DEFINE FIELD added_at       ON shopping_cart TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_cart_owner   ON shopping_cart FIELDS owner;
DEFINE INDEX idx_cart_session ON shopping_cart FIELDS session_id;

DEFINE TABLE payment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD order          ON payment TYPE record<store_order>;
DEFINE FIELD amount         ON payment TYPE float;
DEFINE FIELD currency       ON payment TYPE string VALUE $value OR "SAR";
DEFINE FIELD method         ON payment TYPE string
    ASSERT $value IN ["cash", "bank_transfer", "credit_card", "mada", "stc_pay", "apple_pay", "other"]
    VALUE $value OR "cash";
DEFINE FIELD status         ON payment TYPE string
    ASSERT $value IN ["pending", "completed", "failed", "refunded"]
    VALUE $value OR "pending";
DEFINE FIELD external_ref   ON payment TYPE option<string>;
DEFINE FIELD gateway_response ON payment TYPE option<object>;
DEFINE FIELD paid_at        ON payment TYPE option<datetime>;
DEFINE FIELD notes          ON payment TYPE option<string>;
DEFINE FIELD created_at     ON payment TYPE datetime VALUE $value OR time::now();

DEFINE INDEX idx_payment_order  ON payment FIELDS order;
DEFINE INDEX idx_payment_status ON payment FIELDS status;
