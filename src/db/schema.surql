-- ============================================================================
-- Dr.Machine ERP — SurrealDB Schema (Production)
-- النظام المتكامل لإدارة الموارد البشرية وعلاقات العملاء والعمليات
-- ============================================================================
-- ملاحظة: NS/DB يتم اختيارها عبر .env — لا تُعرّف هنا

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١: الهوية والصلاحيات (Identity & RBAC)           ██
-- ████████████████████████████████████████████████████████████████

DEFINE TABLE account SCHEMAFULL
    PERMISSIONS
        FOR select WHERE id = $auth.id OR $auth.role = "admin"
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD email         ON account TYPE string   ASSERT string::is::email($value);
DEFINE FIELD password_hash ON account TYPE string;
DEFINE FIELD role          ON account TYPE string
    ASSERT $value IN ["admin", "manager", "employee", "intern", "applicant"]
    DEFAULT "applicant";
DEFINE FIELD is_active     ON account TYPE bool     DEFAULT true;
DEFINE FIELD auth_methods  ON account TYPE array    DEFAULT [];
DEFINE FIELD created_at    ON account TYPE datetime DEFAULT time::now();
DEFINE FIELD last_login    ON account TYPE option<datetime>;

DEFINE INDEX idx_account_email ON account FIELDS email UNIQUE;

-- Email verification & password reset
DEFINE FIELD email_verified ON account TYPE bool DEFAULT false;
DEFINE FIELD full_name     ON account TYPE option<string>;
DEFINE FIELD phone         ON account TYPE option<string>;
DEFINE FIELD user_type     ON account TYPE string
    ASSERT $value IN ["platform_admin", "startup_owner", "team_leader", "member", "trainee", "volunteer"]
    DEFAULT "member";
DEFINE FIELD organization  ON account TYPE option<record<organization>>;

DEFINE TABLE auth_token SCHEMAFULL
    PERMISSIONS FOR select, create, update, delete NONE;

DEFINE FIELD account       ON auth_token TYPE record<account>;
DEFINE FIELD token         ON auth_token TYPE string;
DEFINE FIELD token_type    ON auth_token TYPE string
    ASSERT $value IN ["email_verification", "password_reset"];
DEFINE FIELD expires_at    ON auth_token TYPE datetime;
DEFINE FIELD used          ON auth_token TYPE bool DEFAULT false;
DEFINE FIELD created_at    ON auth_token TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_auth_token ON auth_token FIELDS token UNIQUE;

-- Organization Registration Request
DEFINE TABLE org_request SCHEMAFULL
    PERMISSIONS FOR select, update, delete WHERE $auth.role = "admin"
    FOR create NONE;

DEFINE FIELD name           ON org_request TYPE string;
DEFINE FIELD org_type       ON org_request TYPE string DEFAULT "startup";
DEFINE FIELD contact_email  ON org_request TYPE string;
DEFINE FIELD contact_phone  ON org_request TYPE option<string>;
DEFINE FIELD industry       ON org_request TYPE option<string>;
DEFINE FIELD city           ON org_request TYPE option<string>;
DEFINE FIELD website        ON org_request TYPE option<string>;
DEFINE FIELD description    ON org_request TYPE option<string>;
DEFINE FIELD team_size      ON org_request TYPE int DEFAULT 1;
DEFINE FIELD status         ON org_request TYPE string
    ASSERT $value IN ["pending", "approved", "rejected"]
    DEFAULT "pending";
DEFINE FIELD requested_by   ON org_request TYPE option<record<account>>;
DEFINE FIELD reviewed_by    ON org_request TYPE option<record<account>>;
DEFINE FIELD created_at     ON org_request TYPE datetime DEFAULT time::now();
DEFINE FIELD reviewed_at    ON org_request TYPE option<datetime>;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٢: الموارد البشرية (HR)                          ██
-- ████████████████████████████████████████████████████████████████

-- Organization (Startup / Volunteer Team)
DEFINE TABLE organization SCHEMAFULL
    PERMISSIONS
        FOR select WHERE owner = $auth.id OR $auth.role = "admin" OR id IN (SELECT organization FROM account WHERE id = $auth.id)
        FOR create NONE
        FOR update WHERE owner = $auth.id OR $auth.role = "admin"
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD name          ON organization TYPE string ASSERT string::len($value) > 1;
DEFINE FIELD org_type      ON organization TYPE string
    ASSERT $value IN ["startup", "volunteer_team"]
    DEFAULT "startup";
DEFINE FIELD owner         ON organization TYPE record<account>;
DEFINE FIELD industry      ON organization TYPE option<string>;
DEFINE FIELD stage         ON organization TYPE option<string>;
DEFINE FIELD description   ON organization TYPE option<string>;
DEFINE FIELD city          ON organization TYPE option<string>;
DEFINE FIELD address       ON organization TYPE option<string>;
DEFINE FIELD website       ON organization TYPE option<string>;
DEFINE FIELD logo          ON organization TYPE option<string>;
DEFINE FIELD member_count  ON organization TYPE int DEFAULT 1;
DEFINE FIELD status        ON organization TYPE string
    ASSERT $value IN ["pending", "approved", "suspended", "rejected"]
    DEFAULT "pending";
DEFINE FIELD is_public     ON organization TYPE bool DEFAULT false;
DEFINE FIELD created_at    ON organization TYPE datetime DEFAULT time::now();
DEFINE FIELD approved_at   ON organization TYPE option<datetime>;

DEFINE INDEX idx_org_owner ON organization FIELDS owner;
DEFINE INDEX idx_org_status ON organization FIELDS status;

-- employment_contract
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
DEFINE FIELD transport_allowance ON employment_contract TYPE option<float>;
DEFINE FIELD work_location     ON employment_contract TYPE string
    ASSERT $value IN ["onsite", "remote", "hybrid"];
DEFINE FIELD is_active         ON employment_contract TYPE bool  DEFAULT true;
DEFINE FIELD created_at        ON employment_contract TYPE datetime DEFAULT time::now();

-- salary
DEFINE TABLE salary SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee       ON salary TYPE record<account>;
DEFINE FIELD base_amount    ON salary TYPE float  ASSERT $value >= 0;
DEFINE FIELD currency       ON salary TYPE string   DEFAULT "SAR";
DEFINE FIELD effective_from ON salary TYPE datetime;
DEFINE FIELD effective_to   ON salary TYPE option<datetime>;
DEFINE FIELD notes          ON salary TYPE option<string>;

-- payroll
DEFINE TABLE payroll SCHEMAFULL
    PERMISSIONS
        FOR select, create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee        ON payroll TYPE record<account>;
DEFINE FIELD period_month    ON payroll TYPE int    ASSERT $value >= 1 AND $value <= 12;
DEFINE FIELD period_year     ON payroll TYPE int    ASSERT $value >= 2020;
DEFINE FIELD base_salary     ON payroll TYPE float;
DEFINE FIELD housing_allowance ON payroll TYPE option<float>;
DEFINE FIELD transport_allowance ON payroll TYPE option<float>;
DEFINE FIELD other_allowances ON payroll TYPE option<float>;
DEFINE FIELD overtime_hours  ON payroll TYPE option<float>;
DEFINE FIELD overtime_amount ON payroll TYPE option<float>;
DEFINE FIELD deductions      ON payroll TYPE option<float>;
DEFINE FIELD deduction_notes ON payroll TYPE option<string>;
DEFINE FIELD gosi_employee   ON payroll TYPE option<float>;
DEFINE FIELD gosi_employer   ON payroll TYPE option<float>;
DEFINE FIELD net_salary      ON payroll TYPE float;
DEFINE FIELD currency        ON payroll TYPE string  DEFAULT "SAR";
DEFINE FIELD payment_method  ON payroll TYPE string
    ASSERT $value IN ["bank_transfer", "cash", "check"]
    DEFAULT "bank_transfer";
DEFINE FIELD payment_date    ON payroll TYPE option<datetime>;
DEFINE FIELD status          ON payroll TYPE string
    ASSERT $value IN ["draft", "approved", "paid", "cancelled"]
    DEFAULT "draft";
DEFINE FIELD approved_by     ON payroll TYPE option<record<account>>;
DEFINE FIELD notes           ON payroll TYPE option<string>;
DEFINE FIELD created_at      ON payroll TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_payroll_emp_period ON payroll FIELDS employee, period_year, period_month UNIQUE;
DEFINE INDEX idx_payroll_status     ON payroll FIELDS status;

-- employee (simplified for current UI)
DEFINE TABLE employee SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD name         ON employee TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD phone        ON employee TYPE string;
DEFINE FIELD email        ON employee TYPE option<string>;
DEFINE FIELD role         ON employee TYPE string;
DEFINE FIELD national_id  ON employee TYPE option<string>;
DEFINE FIELD hire_date    ON employee TYPE string;
DEFINE FIELD status       ON employee TYPE string
    ASSERT $value IN ["active", "onleave", "resigned"]
    DEFAULT "active";
DEFINE FIELD created_at   ON employee TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD organization ON employee TYPE option<record<organization>>;

-- attendance
DEFINE TABLE attendance SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE $auth.role = "admin";

DEFINE FIELD person_id    ON attendance TYPE string;
DEFINE FIELD person_type  ON attendance TYPE string ASSERT $value IN ["employee", "trainee"];
DEFINE FIELD person_name  ON attendance TYPE option<string>;
DEFINE FIELD check_in     ON attendance TYPE string;
DEFINE FIELD check_out    ON attendance TYPE option<string>;
DEFINE FIELD date         ON attendance TYPE option<string>;
DEFINE FIELD notes        ON attendance TYPE option<string>;

-- leave_request
DEFINE TABLE leave_request SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee    ON leave_request TYPE record<account>;
DEFINE FIELD leave_type  ON leave_request TYPE string
    ASSERT $value IN [
        "annual", "sick", "unpaid", "emergency", "maternity",
        "paternity", "bereavement", "hajj", "marriage",
        "study", "compensatory"
    ];
DEFINE FIELD start_date  ON leave_request TYPE datetime;
DEFINE FIELD end_date    ON leave_request TYPE datetime;
DEFINE FIELD days        ON leave_request TYPE int   ASSERT $value > 0;
DEFINE FIELD reason      ON leave_request TYPE option<string>;
DEFINE FIELD status      ON leave_request TYPE string
    ASSERT $value IN ["pending", "approved", "rejected", "cancelled"]
    DEFAULT "pending";
DEFINE FIELD approved_by ON leave_request TYPE option<record<account>>;
DEFINE FIELD created_at  ON leave_request TYPE datetime DEFAULT time::now();

-- leave_balance
DEFINE TABLE leave_balance SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD employee      ON leave_balance TYPE record<account>;
DEFINE FIELD year           ON leave_balance TYPE int;
DEFINE FIELD annual_total   ON leave_balance TYPE int   DEFAULT 21;
DEFINE FIELD annual_used    ON leave_balance TYPE int   DEFAULT 0;
DEFINE FIELD sick_total     ON leave_balance TYPE int   DEFAULT 10;
DEFINE FIELD sick_used      ON leave_balance TYPE int   DEFAULT 0;
DEFINE FIELD emergency_total ON leave_balance TYPE int  DEFAULT 5;
DEFINE FIELD emergency_used ON leave_balance TYPE int   DEFAULT 0;

DEFINE INDEX idx_leave_balance_emp_year ON leave_balance FIELDS employee, year UNIQUE;

-- evaluation
DEFINE TABLE evaluation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee       ON evaluation TYPE record<account>;
DEFINE FIELD evaluator      ON evaluation TYPE record<account>;
DEFINE FIELD period         ON evaluation TYPE string
    ASSERT $value IN ["q1", "q2", "q3", "q4", "annual"]
    DEFAULT "annual";
DEFINE FIELD year            ON evaluation TYPE int;
DEFINE FIELD productivity    ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD quality         ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD teamwork        ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD initiative      ON evaluation TYPE int   ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD attendance_score ON evaluation TYPE int  ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD overall_score   ON evaluation TYPE float;
DEFINE FIELD strengths       ON evaluation TYPE option<string>;
DEFINE FIELD improvements    ON evaluation TYPE option<string>;
DEFINE FIELD goals           ON evaluation TYPE array  DEFAULT [];
DEFINE FIELD employee_notes  ON evaluation TYPE option<string>;
DEFINE FIELD status          ON evaluation TYPE string
    ASSERT $value IN ["draft", "submitted", "acknowledged"]
    DEFAULT "draft";
DEFINE FIELD created_at      ON evaluation TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_eval_emp_period ON evaluation FIELDS employee, year, period UNIQUE;

-- log (RFID raw)
DEFINE TABLE log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "security"]
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD rfid_uid    ON log TYPE string;
DEFINE FIELD device_id   ON log TYPE string;
DEFINE FIELD timestamp   ON log TYPE datetime DEFAULT time::now();
DEFINE FIELD action      ON log TYPE string;
DEFINE FIELD raw_payload ON log TYPE option<object>;

-- trainee
DEFINE TABLE trainee SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD name         ON trainee TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD phone        ON trainee TYPE string;
DEFINE FIELD institution  ON trainee TYPE string;
DEFINE FIELD start_date   ON trainee TYPE string;
DEFINE FIELD end_date     ON trainee TYPE string;
DEFINE FIELD supervisor   ON trainee TYPE option<record<account>>;
DEFINE FIELD status       ON trainee TYPE string
    ASSERT $value IN ["active", "completed", "cancelled"]
    DEFAULT "active";
DEFINE FIELD notes        ON trainee TYPE option<string>;
DEFINE FIELD created_at   ON trainee TYPE option<datetime> DEFAULT time::now();

-- certificate
DEFINE TABLE certificate SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD credential_id   ON certificate TYPE string;
DEFINE FIELD trainee         ON certificate TYPE option<record<trainee>>;
DEFINE FIELD trainee_name    ON certificate TYPE string;
DEFINE FIELD course_title    ON certificate TYPE string;
DEFINE FIELD total_hours     ON certificate TYPE int     ASSERT $value > 0;
DEFINE FIELD start_date      ON certificate TYPE string;
DEFINE FIELD end_date        ON certificate TYPE string;
DEFINE FIELD instructor      ON certificate TYPE string;
DEFINE FIELD issued_date     ON certificate TYPE string;
DEFINE FIELD expiry_date     ON certificate TYPE option<string>;
DEFINE FIELD status          ON certificate TYPE string
    ASSERT $value IN ["issued", "revoked", "expired"]
    DEFAULT "issued";
DEFINE FIELD verification_hash ON certificate TYPE option<string>;
DEFINE FIELD pdf_url          ON certificate TYPE option<string>;
DEFINE FIELD qr_data          ON certificate TYPE option<string>;
DEFINE FIELD notes            ON certificate TYPE option<string>;

DEFINE INDEX idx_cert_credential ON certificate FIELDS credential_id UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٣: إدارة علاقات العملاء (CRM)                    ██
-- ████████████████████████████████████████████████████████████████

-- client
DEFINE TABLE client SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD company_name    ON client TYPE string   ASSERT string::len($value) > 0;
DEFINE FIELD contact_person  ON client TYPE option<string>;
DEFINE FIELD email           ON client TYPE option<string>;
DEFINE FIELD phone           ON client TYPE option<string>;
DEFINE FIELD address         ON client TYPE option<string>;
DEFINE FIELD city            ON client TYPE option<string>;
DEFINE FIELD latitude        ON client TYPE option<float>;
DEFINE FIELD longitude       ON client TYPE option<float>;
DEFINE FIELD industry        ON client TYPE option<string>;
DEFINE FIELD credit_limit    ON client TYPE float   DEFAULT 0.0;
DEFINE FIELD status          ON client TYPE string
    ASSERT $value IN ["active", "inactive", "pending"]
    DEFAULT "pending";
DEFINE FIELD notes           ON client TYPE option<string>;
DEFINE FIELD created_at      ON client TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD organization    ON client TYPE option<record<organization>>;

-- contact
DEFINE TABLE contact SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client      ON contact TYPE record<client>;
DEFINE FIELD name        ON contact TYPE string   ASSERT string::len($value) > 1;
DEFINE FIELD title       ON contact TYPE option<string>;
DEFINE FIELD email       ON contact TYPE option<string>;
DEFINE FIELD phone       ON contact TYPE option<string>;
DEFINE FIELD is_primary  ON contact TYPE bool     DEFAULT false;
DEFINE FIELD notes       ON contact TYPE option<string>;
DEFINE FIELD created_at  ON contact TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_contact_client ON contact FIELDS client;

-- interaction
DEFINE TABLE interaction SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE employee = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;

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
DEFINE FIELD created_at    ON interaction TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_interaction_client ON interaction FIELDS client, created_at;

-- opportunity
DEFINE TABLE opportunity SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client         ON opportunity TYPE record<client>;
DEFINE FIELD title          ON opportunity TYPE string;
DEFINE FIELD description    ON opportunity TYPE option<string>;
DEFINE FIELD value          ON opportunity TYPE float  ASSERT $value >= 0;
DEFINE FIELD currency       ON opportunity TYPE string   DEFAULT "SAR";
DEFINE FIELD stage          ON opportunity TYPE string
    ASSERT $value IN ["lead", "qualified", "proposal", "negotiation", "won", "lost"]
    DEFAULT "lead";
DEFINE FIELD probability    ON opportunity TYPE int
    ASSERT $value >= 0 AND $value <= 100
    DEFAULT 10;
DEFINE FIELD expected_close ON opportunity TYPE option<datetime>;
DEFINE FIELD assigned_to    ON opportunity TYPE option<record<account>>;
DEFINE FIELD lost_reason    ON opportunity TYPE option<string>;
DEFINE FIELD notes          ON opportunity TYPE option<string>;
DEFINE FIELD created_at     ON opportunity TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at     ON opportunity TYPE datetime VALUE time::now();

DEFINE INDEX idx_opportunity_client ON opportunity FIELDS client;
DEFINE INDEX idx_opportunity_stage  ON opportunity FIELDS stage;

-- quotation
DEFINE TABLE quotation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client         ON quotation TYPE record<client>;
DEFINE FIELD opportunity    ON quotation TYPE option<record<opportunity>>;
DEFINE FIELD quote_number   ON quotation TYPE string;
DEFINE FIELD items          ON quotation TYPE array DEFAULT [];
DEFINE FIELD subtotal       ON quotation TYPE float  ASSERT $value >= 0;
DEFINE FIELD tax_rate       ON quotation TYPE float  DEFAULT 15.0;
DEFINE FIELD tax_amount     ON quotation TYPE float;
DEFINE FIELD total          ON quotation TYPE float;
DEFINE FIELD validity_date  ON quotation TYPE option<datetime>;
DEFINE FIELD status         ON quotation TYPE string
    ASSERT $value IN ["draft", "sent", "accepted", "rejected", "expired"]
    DEFAULT "draft";
DEFINE FIELD notes          ON quotation TYPE option<string>;
DEFINE FIELD created_at     ON quotation TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_quotation_number ON quotation FIELDS quote_number UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٤: المالية (Finance)                              ██
-- ████████████████████████████████████████████████████████████████

-- invoice
DEFINE TABLE invoice SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client          ON invoice TYPE option<record<client>>;
DEFINE FIELD invoice_number  ON invoice TYPE string;
DEFINE FIELD client_id       ON invoice TYPE option<string>;
DEFINE FIELD client_name     ON invoice TYPE option<string>;
DEFINE FIELD invoice_date    ON invoice TYPE string;
DEFINE FIELD due_date        ON invoice TYPE option<string>;
DEFINE FIELD items           ON invoice TYPE array DEFAULT [];
DEFINE FIELD subtotal        ON invoice TYPE float  ASSERT $value >= 0;
DEFINE FIELD tax_rate        ON invoice TYPE float  DEFAULT 15.0;
DEFINE FIELD tax_amount      ON invoice TYPE float;
DEFINE FIELD total           ON invoice TYPE float;
DEFINE FIELD paid_amount     ON invoice TYPE float  DEFAULT 0.0;
DEFINE FIELD balance         ON invoice TYPE float;
DEFINE FIELD currency        ON invoice TYPE string   DEFAULT "SAR";
DEFINE FIELD status          ON invoice TYPE string
    ASSERT $value IN ["draft", "sent", "partially_paid", "paid", "overdue", "cancelled"]
    DEFAULT "draft";
DEFINE FIELD notes           ON invoice TYPE option<string>;
DEFINE FIELD created_at      ON invoice TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD organization    ON invoice TYPE option<record<organization>>;

DEFINE INDEX idx_invoice_number ON invoice FIELDS invoice_number UNIQUE;

-- payment (Immutable)
DEFINE TABLE payment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;

DEFINE FIELD invoice          ON payment TYPE record<invoice>;
DEFINE FIELD amount           ON payment TYPE float  ASSERT $value > 0;
DEFINE FIELD currency         ON payment TYPE string   DEFAULT "SAR";
DEFINE FIELD payment_date     ON payment TYPE datetime DEFAULT time::now();
DEFINE FIELD method           ON payment TYPE string
    ASSERT $value IN ["cash", "bank_transfer", "check", "credit_card", "online"];
DEFINE FIELD reference_number ON payment TYPE option<string>;
DEFINE FIELD received_by      ON payment TYPE option<record<account>>;
DEFINE FIELD notes            ON payment TYPE option<string>;
DEFINE FIELD created_at       ON payment TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_payment_invoice ON payment FIELDS invoice;

-- expense
DEFINE TABLE expense SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee     ON expense TYPE record<account>;
DEFINE FIELD category     ON expense TYPE string
    ASSERT $value IN ["supplies", "utilities", "salary", "equipment", "travel", "maintenance", "rent", "communication", "other"];
DEFINE FIELD amount       ON expense TYPE float  ASSERT $value > 0;
DEFINE FIELD currency     ON expense TYPE string   DEFAULT "SAR";
DEFINE FIELD date         ON expense TYPE datetime;
DEFINE FIELD description  ON expense TYPE string;
DEFINE FIELD receipt_url  ON expense TYPE option<string>;
DEFINE FIELD approved_by  ON expense TYPE option<record<account>>;
DEFINE FIELD status       ON expense TYPE string
    ASSERT $value IN ["pending", "approved", "rejected"]
    DEFAULT "pending";
DEFINE FIELD project      ON expense TYPE option<record<project>>;
DEFINE FIELD notes        ON expense TYPE option<string>;
DEFINE FIELD created_at   ON expense TYPE datetime DEFAULT time::now();
DEFINE FIELD organization ON expense TYPE option<record<organization>>;

DEFINE INDEX idx_expense_date ON expense FIELDS date;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٤.٢: المشتريات وسلاسل الإمداد (Procurement)      ██
-- ████████████████████████████████████████████████████████████████

-- vendor
DEFINE TABLE vendor SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD company_name    ON vendor TYPE string;
DEFINE FIELD contact_person  ON vendor TYPE option<string>;
DEFINE FIELD email           ON vendor TYPE option<string>;
DEFINE FIELD phone           ON vendor TYPE option<string>;
DEFINE FIELD address         ON vendor TYPE option<string>;
DEFINE FIELD city            ON vendor TYPE option<string>;
DEFINE FIELD country         ON vendor TYPE string  DEFAULT "SA";
DEFINE FIELD tax_number      ON vendor TYPE option<string>;
DEFINE FIELD payment_terms   ON vendor TYPE string
    ASSERT $value IN ["cash", "net_15", "net_30", "net_60", "net_90"]
    DEFAULT "net_30";
DEFINE FIELD currency        ON vendor TYPE string  DEFAULT "SAR";
DEFINE FIELD bank_name       ON vendor TYPE option<string>;
DEFINE FIELD bank_iban       ON vendor TYPE option<string>;
DEFINE FIELD rating          ON vendor TYPE option<int>;
DEFINE FIELD categories      ON vendor TYPE array  DEFAULT [];
DEFINE FIELD status          ON vendor TYPE string
    ASSERT $value IN ["active", "inactive", "blacklisted"]
    DEFAULT "active";
DEFINE FIELD notes           ON vendor TYPE option<string>;
DEFINE FIELD created_at      ON vendor TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_vendor_name ON vendor FIELDS company_name;

-- purchase_order
DEFINE TABLE purchase_order SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD po_number       ON purchase_order TYPE string;
DEFINE FIELD vendor          ON purchase_order TYPE record<vendor>;
DEFINE FIELD requested_by    ON purchase_order TYPE record<account>;
DEFINE FIELD approved_by     ON purchase_order TYPE option<record<account>>;
DEFINE FIELD order_date      ON purchase_order TYPE datetime DEFAULT time::now();
DEFINE FIELD expected_date   ON purchase_order TYPE option<datetime>;
DEFINE FIELD items           ON purchase_order TYPE array  DEFAULT [];
DEFINE FIELD subtotal        ON purchase_order TYPE float;
DEFINE FIELD tax_rate        ON purchase_order TYPE option<float>;
DEFINE FIELD tax_amount      ON purchase_order TYPE float;
DEFINE FIELD total           ON purchase_order TYPE float;
DEFINE FIELD currency        ON purchase_order TYPE string  DEFAULT "SAR";
DEFINE FIELD status          ON purchase_order TYPE string
    ASSERT $value IN ["draft", "pending_approval", "approved", "ordered", "partially_received", "received", "cancelled"]
    DEFAULT "draft";
DEFINE FIELD notes           ON purchase_order TYPE option<string>;
DEFINE FIELD created_at      ON purchase_order TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_po_number ON purchase_order FIELDS po_number UNIQUE;
DEFINE INDEX idx_po_vendor ON purchase_order FIELDS vendor;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٥: الأصول والمخزون (Assets & Inventory)          ██
-- ████████████████████████████████████████████████████████████████

-- asset
DEFINE TABLE asset SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD name           ON asset TYPE string;
DEFINE FIELD category       ON asset TYPE string;
DEFINE FIELD serial_number  ON asset TYPE option<string>;
DEFINE FIELD purchase_date  ON asset TYPE option<string>;
DEFINE FIELD value          ON asset TYPE option<float>;
DEFINE FIELD depreciation   ON asset TYPE float     DEFAULT 0.0;
DEFINE FIELD location       ON asset TYPE option<string>;
DEFINE FIELD assigned_to    ON asset TYPE option<string>;
DEFINE FIELD assigned_employee_name ON asset TYPE option<string>;
DEFINE FIELD status         ON asset TYPE string
    ASSERT $value IN ["available", "inuse", "maintenance", "disposed", "lost"]
    DEFAULT "available";
DEFINE FIELD notes          ON asset TYPE option<string>;
DEFINE FIELD created_at     ON asset TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD organization   ON asset TYPE option<record<organization>>;

-- machine
DEFINE TABLE machine SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD customer_id     ON machine TYPE option<string>;
DEFINE FIELD customer_name   ON machine TYPE option<string>;
DEFINE FIELD serial_number   ON machine TYPE string;
DEFINE FIELD model           ON machine TYPE string;
DEFINE FIELD manufacturer    ON machine TYPE string;
DEFINE FIELD purchase_date   ON machine TYPE option<string>;
DEFINE FIELD warranty_end    ON machine TYPE option<string>;
DEFINE FIELD status          ON machine TYPE string
    ASSERT $value IN ["working", "broken", "repairing", "sold"]
    DEFAULT "working";
DEFINE FIELD location        ON machine TYPE option<string>;
DEFINE FIELD notes           ON machine TYPE option<string>;
DEFINE FIELD created_at      ON machine TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD organization    ON machine TYPE option<record<organization>>;

DEFINE INDEX idx_machine_serial ON machine FIELDS serial_number UNIQUE;

-- inventory_item
DEFINE TABLE inventory_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD sku           ON inventory_item TYPE string;
DEFINE FIELD name          ON inventory_item TYPE string;
DEFINE FIELD category      ON inventory_item TYPE string
    ASSERT $value IN ["spare_parts", "raw_materials", "consumables", "packaging", "other"];
DEFINE FIELD unit          ON inventory_item TYPE string DEFAULT "piece";
DEFINE FIELD quantity      ON inventory_item TYPE int     DEFAULT 0;
DEFINE FIELD min_quantity  ON inventory_item TYPE int     DEFAULT 5;
DEFINE FIELD unit_cost     ON inventory_item TYPE option<float>;
DEFINE FIELD currency      ON inventory_item TYPE string  DEFAULT "SAR";
DEFINE FIELD location      ON inventory_item TYPE option<string>;
DEFINE FIELD supplier      ON inventory_item TYPE option<string>;
DEFINE FIELD notes         ON inventory_item TYPE option<string>;
DEFINE FIELD created_at    ON inventory_item TYPE datetime DEFAULT time::now();
DEFINE FIELD organization  ON inventory_item TYPE option<record<organization>>;

DEFINE INDEX idx_inventory_sku ON inventory_item FIELDS sku UNIQUE;

-- inventory_transaction (Immutable)
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
DEFINE FIELD created_at  ON inventory_transaction TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_invtx_item ON inventory_transaction FIELDS item, created_at;

-- warehouse
DEFINE TABLE warehouse SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD name          ON warehouse TYPE string  ASSERT string::len($value) > 0;
DEFINE FIELD code          ON warehouse TYPE string;
DEFINE FIELD address       ON warehouse TYPE option<string>;
DEFINE FIELD manager       ON warehouse TYPE option<record<account>>;
DEFINE FIELD is_active     ON warehouse TYPE bool    DEFAULT true;
DEFINE FIELD capacity      ON warehouse TYPE option<int>;
DEFINE FIELD notes         ON warehouse TYPE option<string>;
DEFINE FIELD created_at    ON warehouse TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_warehouse_code ON warehouse FIELDS code UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٦: الصيانة والخدمات (Repairs & Services)         ██
-- ████████████████████████████████████████████████████████████████

-- repair_operation
DEFINE TABLE repair_operation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE false;

DEFINE FIELD machine_id     ON repair_operation TYPE option<string>;
DEFINE FIELD machine_serial ON repair_operation TYPE option<string>;
DEFINE FIELD project_id     ON repair_operation TYPE option<string>;
DEFINE FIELD project_title  ON repair_operation TYPE option<string>;
DEFINE FIELD employee_id    ON repair_operation TYPE option<string>;
DEFINE FIELD employee_name  ON repair_operation TYPE option<string>;
DEFINE FIELD customer_name  ON repair_operation TYPE option<string>;
DEFINE FIELD description    ON repair_operation TYPE string;
DEFINE FIELD diagnosis      ON repair_operation TYPE option<string>;
DEFINE FIELD parts_used     ON repair_operation TYPE option<string>;
DEFINE FIELD labor_hours    ON repair_operation TYPE option<float>;
DEFINE FIELD cost           ON repair_operation TYPE option<float>;
DEFINE FIELD warranty_claim ON repair_operation TYPE bool    DEFAULT false;
DEFINE FIELD start_time     ON repair_operation TYPE string;
DEFINE FIELD end_time       ON repair_operation TYPE option<string>;
DEFINE FIELD status         ON repair_operation TYPE string
    ASSERT $value IN ["new", "diagnosing", "repairing", "waiting", "completed", "cancelled"]
    DEFAULT "new";
DEFINE FIELD notes          ON repair_operation TYPE option<string>;
DEFINE FIELD created_at     ON repair_operation TYPE option<datetime> DEFAULT time::now();

DEFINE INDEX idx_repair_status  ON repair_operation FIELDS status;

-- service_contract
DEFINE TABLE service_contract SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client         ON service_contract TYPE record<client>;
DEFINE FIELD machines       ON service_contract TYPE array DEFAULT [];
DEFINE FIELD title          ON service_contract TYPE string;
DEFINE FIELD start_date     ON service_contract TYPE datetime;
DEFINE FIELD end_date       ON service_contract TYPE datetime;
DEFINE FIELD value          ON service_contract TYPE float  ASSERT $value >= 0;
DEFINE FIELD currency       ON service_contract TYPE string   DEFAULT "SAR";
DEFINE FIELD frequency      ON service_contract TYPE string
    ASSERT $value IN ["monthly", "quarterly", "semi_annual", "annual"];
DEFINE FIELD next_service   ON service_contract TYPE option<datetime>;
DEFINE FIELD status         ON service_contract TYPE string
    ASSERT $value IN ["active", "expired", "suspended", "cancelled"]
    DEFAULT "active";
DEFINE FIELD notes          ON service_contract TYPE option<string>;
DEFINE FIELD created_at     ON service_contract TYPE datetime DEFAULT time::now();

-- warranty_card
DEFINE TABLE warranty_card SCHEMAFULL
    PERMISSIONS
        FOR select WHERE true
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD warranty_number  ON warranty_card TYPE string;
DEFINE FIELD client_name      ON warranty_card TYPE option<string>;
DEFINE FIELD product_name     ON warranty_card TYPE string;
DEFINE FIELD serial_number    ON warranty_card TYPE option<string>;
DEFINE FIELD model            ON warranty_card TYPE option<string>;
DEFINE FIELD purchase_date    ON warranty_card TYPE datetime;
DEFINE FIELD warranty_start   ON warranty_card TYPE datetime;
DEFINE FIELD warranty_end     ON warranty_card TYPE datetime;
DEFINE FIELD warranty_months  ON warranty_card TYPE int    ASSERT $value > 0;
DEFINE FIELD coverage_type    ON warranty_card TYPE string
    ASSERT $value IN ["full", "parts_only", "labor_only", "limited"]
    DEFAULT "full";
DEFINE FIELD terms            ON warranty_card TYPE option<string>;
DEFINE FIELD exclusions       ON warranty_card TYPE option<string>;
DEFINE FIELD max_claims       ON warranty_card TYPE option<int>;
DEFINE FIELD claims_used      ON warranty_card TYPE int  DEFAULT 0;
DEFINE FIELD status           ON warranty_card TYPE string
    ASSERT $value IN ["active", "expired", "voided", "claimed"]
    DEFAULT "active";
DEFINE FIELD verification_hash ON warranty_card TYPE option<string>;
DEFINE FIELD qr_data           ON warranty_card TYPE option<string>;
DEFINE FIELD notes             ON warranty_card TYPE option<string>;
DEFINE FIELD created_at        ON warranty_card TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_warranty_number ON warranty_card FIELDS warranty_number UNIQUE;

-- maintenance_schedule
DEFINE TABLE maintenance_schedule SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD machine          ON maintenance_schedule TYPE record<machine>;
DEFINE FIELD title            ON maintenance_schedule TYPE string;
DEFINE FIELD description      ON maintenance_schedule TYPE option<string>;
DEFINE FIELD frequency_type   ON maintenance_schedule TYPE string
    ASSERT $value IN ["calendar", "hours", "cycles"]
    DEFAULT "calendar";
DEFINE FIELD frequency_value  ON maintenance_schedule TYPE int   ASSERT $value > 0;
DEFINE FIELD last_performed   ON maintenance_schedule TYPE option<datetime>;
DEFINE FIELD next_due         ON maintenance_schedule TYPE option<datetime>;
DEFINE FIELD assigned_to      ON maintenance_schedule TYPE option<record<account>>;
DEFINE FIELD required_parts   ON maintenance_schedule TYPE array  DEFAULT [];
DEFINE FIELD estimated_hours  ON maintenance_schedule TYPE option<float>;
DEFINE FIELD priority         ON maintenance_schedule TYPE string
    ASSERT $value IN ["low", "medium", "high", "critical"]
    DEFAULT "medium";
DEFINE FIELD is_active        ON maintenance_schedule TYPE bool  DEFAULT true;
DEFINE FIELD notes            ON maintenance_schedule TYPE option<string>;
DEFINE FIELD created_at       ON maintenance_schedule TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_maint_machine ON maintenance_schedule FIELDS machine;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٧: إدارة المشاريع — نمط Trello (Project Boards)  ██
-- ████████████████████████████████████████████████████████████████

-- project (Workspace — يحتوي على عدة لوحات)
DEFINE TABLE project SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD title         ON project TYPE string;
DEFINE FIELD description   ON project TYPE option<string>;
DEFINE FIELD client        ON project TYPE option<record<client>>;
DEFINE FIELD department    ON project TYPE option<record<department>>;
DEFINE FIELD manager       ON project TYPE option<record<account>>;
DEFINE FIELD start_date    ON project TYPE option<datetime>;
DEFINE FIELD end_date      ON project TYPE option<datetime>;
DEFINE FIELD budget        ON project TYPE option<float>;
DEFINE FIELD spent         ON project TYPE option<float>;
DEFINE FIELD currency      ON project TYPE string DEFAULT "SAR";
DEFINE FIELD status        ON project TYPE string
    ASSERT $value IN ["planning", "active", "on_hold", "completed", "cancelled"]
    DEFAULT "planning";
DEFINE FIELD priority      ON project TYPE string
    ASSERT $value IN ["low", "medium", "high", "critical"]
    DEFAULT "medium";
DEFINE FIELD color         ON project TYPE string DEFAULT "#3B82F6";
DEFINE FIELD visibility    ON project TYPE string
    ASSERT $value IN ["private", "team", "public"]
    DEFAULT "team";
DEFINE FIELD tags          ON project TYPE array DEFAULT [];
DEFINE FIELD created_at    ON project TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at    ON project TYPE datetime VALUE time::now();
DEFINE FIELD organization  ON project TYPE option<record<organization>>;

DEFINE INDEX idx_project_status ON project FIELDS status;

-- project_member — أعضاء المشروع
DEFINE TABLE project_member SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD project    ON project_member TYPE record<project>;
DEFINE FIELD member     ON project_member TYPE record<account>;
DEFINE FIELD role       ON project_member TYPE string
    ASSERT $value IN ["owner", "admin", "member", "viewer"]
    DEFAULT "member";
DEFINE FIELD joined_at  ON project_member TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_pm_unique ON project_member FIELDS project, member UNIQUE;

-- board — لوحة كانبان (كل مشروع يمكن أن يملك عدة لوحات)
DEFINE TABLE board SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD project      ON board TYPE record<project>;
DEFINE FIELD title        ON board TYPE string;
DEFINE FIELD description  ON board TYPE option<string>;
DEFINE FIELD background   ON board TYPE string DEFAULT "#1E293B";
DEFINE FIELD is_default   ON board TYPE bool DEFAULT false;
DEFINE FIELD position     ON board TYPE int DEFAULT 0;
DEFINE FIELD created_at   ON board TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_board_project ON board FIELDS project;

-- board_list — أعمدة اللوحة (To Do, In Progress, Done...)
DEFINE TABLE board_list SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD board       ON board_list TYPE record<board>;
DEFINE FIELD title       ON board_list TYPE string;
DEFINE FIELD position    ON board_list TYPE int DEFAULT 0;
DEFINE FIELD color       ON board_list TYPE option<string>;
DEFINE FIELD limit       ON board_list TYPE option<int>;
DEFINE FIELD is_closed   ON board_list TYPE bool DEFAULT false;
DEFINE FIELD created_at  ON board_list TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_list_board ON board_list FIELDS board;

-- card — بطاقة المهمة (العنصر الأساسي — مثل Trello Card)
DEFINE TABLE card SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE false;

DEFINE FIELD board_list   ON card TYPE record<board_list>;
DEFINE FIELD title        ON card TYPE string;
DEFINE FIELD description  ON card TYPE option<string>;
DEFINE FIELD position     ON card TYPE int DEFAULT 0;
DEFINE FIELD priority     ON card TYPE string
    ASSERT $value IN ["none", "low", "medium", "high", "urgent"]
    DEFAULT "none";
DEFINE FIELD due_date     ON card TYPE option<datetime>;
DEFINE FIELD start_date   ON card TYPE option<datetime>;
DEFINE FIELD completed_at ON card TYPE option<datetime>;
DEFINE FIELD estimated_hours ON card TYPE option<float>;
DEFINE FIELD actual_hours ON card TYPE option<float>;
DEFINE FIELD cover_color  ON card TYPE option<string>;
DEFINE FIELD cover_image  ON card TYPE option<string>;
DEFINE FIELD is_complete  ON card TYPE bool DEFAULT false;
DEFINE FIELD created_by   ON card TYPE option<record<account>>;
DEFINE FIELD created_at   ON card TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at   ON card TYPE datetime VALUE time::now();

DEFINE INDEX idx_card_list ON card FIELDS board_list;

-- card_member — أعضاء البطاقة (من يعمل عليها)
DEFINE TABLE card_member SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, delete WHERE $auth.role IN ["admin", "manager", "employee"];

DEFINE FIELD card     ON card_member TYPE record<card>;
DEFINE FIELD member   ON card_member TYPE record<account>;
DEFINE FIELD added_at ON card_member TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cm_unique ON card_member FIELDS card, member UNIQUE;

-- card_label — تسميات البطاقات (ملونة)
DEFINE TABLE card_label SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update, delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD board    ON card_label TYPE record<board>;
DEFINE FIELD name     ON card_label TYPE string;
DEFINE FIELD color    ON card_label TYPE string;
DEFINE FIELD created_at ON card_label TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cl_board ON card_label FIELDS board;

-- card_label_link — ربط بطاقة بتسمية
DEFINE TABLE card_label_link SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, delete WHERE $auth.role IN ["admin", "manager", "employee"];

DEFINE FIELD card   ON card_label_link TYPE record<card>;
DEFINE FIELD label  ON card_label_link TYPE record<card_label>;

DEFINE INDEX idx_cll_unique ON card_label_link FIELDS card, label UNIQUE;

-- card_checklist — قوائم المراجعة داخل البطاقة
DEFINE TABLE card_checklist SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update, delete WHERE $auth.role IN ["admin", "manager", "employee"];

DEFINE FIELD card      ON card_checklist TYPE record<card>;
DEFINE FIELD title     ON card_checklist TYPE string;
DEFINE FIELD position  ON card_checklist TYPE int DEFAULT 0;
DEFINE FIELD created_at ON card_checklist TYPE datetime DEFAULT time::now();

-- checklist_item — عنصر داخل قائمة المراجعة
DEFINE TABLE checklist_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update, delete WHERE $auth.role IN ["admin", "manager", "employee"];

DEFINE FIELD checklist   ON checklist_item TYPE record<card_checklist>;
DEFINE FIELD title       ON checklist_item TYPE string;
DEFINE FIELD is_checked  ON checklist_item TYPE bool DEFAULT false;
DEFINE FIELD position    ON checklist_item TYPE int DEFAULT 0;
DEFINE FIELD assigned_to ON checklist_item TYPE option<record<account>>;
DEFINE FIELD due_date    ON checklist_item TYPE option<datetime>;
DEFINE FIELD created_at  ON checklist_item TYPE datetime DEFAULT time::now();

-- card_comment — تعليقات على البطاقة
DEFINE TABLE card_comment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE author = $auth.id
        FOR delete WHERE author = $auth.id OR $auth.role = "admin";

DEFINE FIELD card       ON card_comment TYPE record<card>;
DEFINE FIELD author     ON card_comment TYPE record<account>;
DEFINE FIELD content    ON card_comment TYPE string;
DEFINE FIELD edited_at  ON card_comment TYPE option<datetime>;
DEFINE FIELD created_at ON card_comment TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cc_card ON card_comment FIELDS card;

-- card_attachment — مرفقات البطاقة
DEFINE TABLE card_attachment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD card       ON card_attachment TYPE record<card>;
DEFINE FIELD file_name  ON card_attachment TYPE string;
DEFINE FIELD file_url   ON card_attachment TYPE string;
DEFINE FIELD file_size  ON card_attachment TYPE int DEFAULT 0;
DEFINE FIELD mime_type  ON card_attachment TYPE option<string>;
DEFINE FIELD uploaded_by ON card_attachment TYPE record<account>;
DEFINE FIELD created_at ON card_attachment TYPE datetime DEFAULT time::now();

-- card_activity — سجل نشاط البطاقة (تلقائي)
DEFINE TABLE card_activity SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee", "system"]
        FOR update, delete WHERE false;

DEFINE FIELD card       ON card_activity TYPE record<card>;
DEFINE FIELD actor      ON card_activity TYPE option<record<account>>;
DEFINE FIELD action     ON card_activity TYPE string
    ASSERT $value IN [
        "created", "moved", "assigned", "unassigned", "completed",
        "reopened", "commented", "label_added", "label_removed",
        "checklist_added", "attachment_added", "due_date_set",
        "priority_changed", "description_updated", "archived"
    ];
DEFINE FIELD details    ON card_activity TYPE option<string>;
DEFINE FIELD created_at ON card_activity TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_ca_card ON card_activity FIELDS card;

-- report
DEFINE TABLE report SCHEMAFULL
    PERMISSIONS
        FOR select WHERE author = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE author = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD author      ON report TYPE record<account>;
DEFINE FIELD title       ON report TYPE string;
DEFINE FIELD content     ON report TYPE string;
DEFINE FIELD report_type ON report TYPE string
    ASSERT $value IN ["daily", "weekly", "incident", "technical", "financial"];
DEFINE FIELD attachments ON report TYPE array DEFAULT [];
DEFINE FIELD created_at  ON report TYPE datetime DEFAULT time::now();

-- job_application
DEFINE TABLE job_application SCHEMAFULL
    PERMISSIONS
        FOR select WHERE applicant = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE true
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD applicant   ON job_application TYPE record<account>;
DEFINE FIELD position    ON job_application TYPE string;
DEFINE FIELD status      ON job_application TYPE string
    ASSERT $value IN ["pending", "reviewing", "interviewed", "accepted", "rejected"]
    DEFAULT "pending";
DEFINE FIELD applied_at  ON job_application TYPE datetime DEFAULT time::now();
DEFINE FIELD notes       ON job_application TYPE option<string>;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٨: الإشعارات والتدقيق (Notifications & Audit)    ██
-- ████████████████████████████████████████████████████████████████

-- notification
DEFINE TABLE notification SCHEMAFULL
    PERMISSIONS
        FOR select WHERE recipient = $auth.id OR $auth.role = "admin"
        FOR create WHERE $auth.role IN ["admin", "system"]
        FOR update WHERE recipient = $auth.id
        FOR delete WHERE recipient = $auth.id OR $auth.role = "admin";

DEFINE FIELD recipient   ON notification TYPE record<account>;
DEFINE FIELD title       ON notification TYPE string;
DEFINE FIELD message     ON notification TYPE string;
DEFINE FIELD notif_type  ON notification TYPE string
    ASSERT $value IN ["info", "warning", "success", "error", "reminder"];
DEFINE FIELD is_read     ON notification TYPE bool     DEFAULT false;
DEFINE FIELD link        ON notification TYPE option<string>;
DEFINE FIELD created_at  ON notification TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_notif_recipient ON notification FIELDS recipient, is_read;

-- audit_log (Immutable — Micro-Blockchain)
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
DEFINE FIELD timestamp   ON audit_log TYPE datetime DEFAULT time::now();
DEFINE FIELD prev_hash   ON audit_log TYPE option<string>;
DEFINE FIELD record_hash ON audit_log TYPE option<string>;

DEFINE INDEX idx_audit_time  ON audit_log FIELDS timestamp;
DEFINE INDEX idx_audit_actor ON audit_log FIELDS actor;

-- document_attachment
DEFINE TABLE document_attachment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE uploaded_by = $auth.id OR $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD file_name       ON document_attachment TYPE string;
DEFINE FIELD file_path       ON document_attachment TYPE string;
DEFINE FIELD file_size       ON document_attachment TYPE int  ASSERT $value > 0;
DEFINE FIELD mime_type       ON document_attachment TYPE string;
DEFINE FIELD category        ON document_attachment TYPE string
    ASSERT $value IN ["contract", "invoice", "receipt", "manual", "photo", "report", "certificate", "id_document", "other"]
    DEFAULT "other";
DEFINE FIELD related_table   ON document_attachment TYPE string;
DEFINE FIELD related_id      ON document_attachment TYPE string;
DEFINE FIELD description     ON document_attachment TYPE option<string>;
DEFINE FIELD tags            ON document_attachment TYPE array  DEFAULT [];
DEFINE FIELD uploaded_by     ON document_attachment TYPE record<account>;
DEFINE FIELD is_public       ON document_attachment TYPE bool  DEFAULT false;
DEFINE FIELD version         ON document_attachment TYPE int   DEFAULT 1;
DEFINE FIELD created_at      ON document_attachment TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_doc_related ON document_attachment FIELDS related_table, related_id;

-- role_permission
DEFINE TABLE role_permission SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role = "admin"
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD role            ON role_permission TYPE string;
DEFINE FIELD resource        ON role_permission TYPE string;
DEFINE FIELD can_create      ON role_permission TYPE bool  DEFAULT false;
DEFINE FIELD can_read        ON role_permission TYPE bool  DEFAULT false;
DEFINE FIELD can_update      ON role_permission TYPE bool  DEFAULT false;
DEFINE FIELD can_delete      ON role_permission TYPE bool  DEFAULT false;
DEFINE FIELD can_export      ON role_permission TYPE bool  DEFAULT false;
DEFINE FIELD is_active       ON role_permission TYPE bool  DEFAULT true;
DEFINE FIELD created_at      ON role_permission TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_role_perm ON role_permission FIELDS role, resource UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ٩: علاقات Graph (Relations)                       ██
-- ████████████████████████████████████████████████████████████████

-- account -> ASSIGNED_TO -> task
DEFINE TABLE assigned_to SCHEMAFULL TYPE RELATION IN account OUT task
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD assigned_at ON assigned_to TYPE datetime DEFAULT time::now();

-- account -> MEMBER_OF -> project
DEFINE TABLE member_of SCHEMAFULL TYPE RELATION IN account OUT project
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD role      ON member_of TYPE string
    ASSERT $value IN ["lead", "member", "observer"] DEFAULT "member";
DEFINE FIELD joined_at ON member_of TYPE datetime DEFAULT time::now();

-- account -> SUBMITTED -> report
DEFINE TABLE submitted SCHEMAFULL TYPE RELATION IN account OUT report
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";

-- account -> CONTACTED -> client
DEFINE TABLE contacted SCHEMAFULL TYPE RELATION IN account OUT client
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";
DEFINE FIELD channel      ON contacted TYPE string;
DEFINE FIELD contacted_at ON contacted TYPE datetime DEFAULT time::now();

-- account -> SUPERVISES -> trainee
DEFINE TABLE supervises SCHEMAFULL TYPE RELATION IN account OUT trainee
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD started_at ON supervises TYPE datetime DEFAULT time::now();

-- machine -> OWNED_BY -> client
DEFINE TABLE owned_by SCHEMAFULL TYPE RELATION IN machine OUT client
    PERMISSIONS FOR create, delete WHERE $auth.role IN ["admin", "manager"];
DEFINE FIELD since ON owned_by TYPE datetime DEFAULT time::now();

-- inventory_item -> USED_IN -> repair_operation
DEFINE TABLE used_in SCHEMAFULL TYPE RELATION IN inventory_item OUT repair_operation
    PERMISSIONS
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE $auth.role = "admin";
DEFINE FIELD quantity ON used_in TYPE int   ASSERT $value > 0;
DEFINE FIELD cost     ON used_in TYPE option<float>;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٠: الأحداث التلقائية (Events)                   ██
-- ████████████████████████████████████████████████████████████████

-- عند قبول طلب توظيف: أنشئ عقد عمل
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

-- عند فوز فرصة: أنشئ مشروع
DEFINE EVENT on_opportunity_won ON opportunity
    WHEN $before.stage != "won" AND $after.stage = "won"
    THEN {
        CREATE project SET
            title       = $after.title,
            description = $after.description OR "",
            status      = "new",
            budget      = $after.value,
            currency    = $after.currency,
            start_date  = time::format(time::now(), "%Y-%m-%d");
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

-- عند اكتمال إصلاح: حدّث حالة الماكينة
DEFINE EVENT on_repair_completed ON repair_operation
    WHEN $before.status != "completed" AND $after.status = "completed"
    THEN {
        LET $mid = $after.machine_id;
        IF $mid != NONE THEN
            UPDATE type::thing("machine", $mid) SET status = "working";
        END;
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

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١١: الحذف المنطقي (Soft Delete — is_archived)    ██
-- ████████████████████████████████████████████████████████████████

DEFINE FIELD is_archived ON leave_request   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON evaluation      TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON trainee         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON certificate     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON client          TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON contact         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON interaction     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON opportunity     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON quotation       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON invoice         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON expense         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON vendor          TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON purchase_order  TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON asset           TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON machine         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON inventory_item  TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON warehouse       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON repair_operation TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON service_contract TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON maintenance_schedule TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON project         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON task            TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON report          TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON job_application TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON notification    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON document_attachment TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON warranty_card   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON payroll         TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٠: النظام المحاسبي القانوني (Accounting System)  ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- company_settings — إعدادات الشركة (هوية ZATCA)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE company_settings SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD company_name      ON company_settings TYPE string;
DEFINE FIELD company_name_en   ON company_settings TYPE option<string>;
DEFINE FIELD vat_number        ON company_settings TYPE string;          -- الرقم الضريبي
DEFINE FIELD cr_number         ON company_settings TYPE option<string>;  -- السجل التجاري
DEFINE FIELD address_ar        ON company_settings TYPE string;
DEFINE FIELD address_en        ON company_settings TYPE option<string>;
DEFINE FIELD city              ON company_settings TYPE string;
DEFINE FIELD country           ON company_settings TYPE string DEFAULT "SA";
DEFINE FIELD postal_code       ON company_settings TYPE option<string>;
DEFINE FIELD phone             ON company_settings TYPE option<string>;
DEFINE FIELD email             ON company_settings TYPE option<string>;
DEFINE FIELD logo_url          ON company_settings TYPE option<string>;
DEFINE FIELD default_currency  ON company_settings TYPE string DEFAULT "SAR";
DEFINE FIELD fiscal_year_start ON company_settings TYPE int DEFAULT 1;  -- شهر بداية السنة المالية
DEFINE FIELD created_at        ON company_settings TYPE datetime DEFAULT time::now();

-- ────────────────────────────────────────────────────────────────
-- account_chart — دليل الحسابات (Chart of Accounts)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE account_chart SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD code       ON account_chart TYPE string;
DEFINE FIELD name       ON account_chart TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD type       ON account_chart TYPE string
    ASSERT $value IN ["asset", "liability", "equity", "revenue", "expense"];
DEFINE FIELD parent     ON account_chart TYPE option<record<account_chart>>;
DEFINE FIELD level      ON account_chart TYPE int DEFAULT 1;
DEFINE FIELD is_active  ON account_chart TYPE bool DEFAULT true;
DEFINE FIELD balance_direction ON account_chart TYPE string
    ASSERT $value IN ["debit", "credit"]
    DEFAULT "debit";
DEFINE FIELD description ON account_chart TYPE option<string>;
DEFINE FIELD is_header  ON account_chart TYPE bool DEFAULT false;
DEFINE FIELD created_at ON account_chart TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_chart_code ON account_chart FIELDS code UNIQUE;
DEFINE INDEX idx_chart_type ON account_chart FIELDS type;

-- ────────────────────────────────────────────────────────────────
-- fiscal_period — الفترات المالية
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE fiscal_period SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD year        ON fiscal_period TYPE int;
DEFINE FIELD month       ON fiscal_period TYPE int ASSERT $value >= 1 AND $value <= 12;
DEFINE FIELD start_date  ON fiscal_period TYPE datetime;
DEFINE FIELD end_date    ON fiscal_period TYPE datetime;
DEFINE FIELD status      ON fiscal_period TYPE string
    ASSERT $value IN ["open", "closed", "locked"]
    DEFAULT "open";
DEFINE FIELD closed_by   ON fiscal_period TYPE option<record<account>>;
DEFINE FIELD closed_at   ON fiscal_period TYPE option<datetime>;
DEFINE FIELD notes       ON fiscal_period TYPE option<string>;
DEFINE FIELD created_at  ON fiscal_period TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_fp_year_month ON fiscal_period FIELDS year, month UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- cost_center — مراكز التكلفة
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE cost_center SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD code        ON cost_center TYPE string;
DEFINE FIELD name        ON cost_center TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD parent      ON cost_center TYPE option<record<cost_center>>;
DEFINE FIELD is_active   ON cost_center TYPE bool DEFAULT true;
DEFINE FIELD created_at  ON cost_center TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cc_code ON cost_center FIELDS code UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- journal_entry — القيود اليومية (غير قابلة للتعديل بعد الترحيل)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE journal_entry SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update WHERE status = "draft" AND $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD entry_number  ON journal_entry TYPE string;
DEFINE FIELD date          ON journal_entry TYPE datetime;
DEFINE FIELD description   ON journal_entry TYPE string;
DEFINE FIELD reference     ON journal_entry TYPE option<string>;
DEFINE FIELD source_type   ON journal_entry TYPE string
    ASSERT $value IN ["invoice", "payment", "expense", "payroll", "adjustment", "opening", "closing", "manual"];
DEFINE FIELD source_id     ON journal_entry TYPE option<string>;
DEFINE FIELD posted_by     ON journal_entry TYPE option<record<account>>;
DEFINE FIELD total_debit   ON journal_entry TYPE option<float>;
DEFINE FIELD total_credit  ON journal_entry TYPE option<float>;
DEFINE FIELD status        ON journal_entry TYPE string
    ASSERT $value IN ["draft", "posted", "reversed"]
    DEFAULT "draft";
DEFINE FIELD fiscal_period ON journal_entry TYPE option<record<fiscal_period>>;
DEFINE FIELD notes         ON journal_entry TYPE option<string>;
DEFINE FIELD created_at    ON journal_entry TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_je_number ON journal_entry FIELDS entry_number UNIQUE;
DEFINE INDEX idx_je_date   ON journal_entry FIELDS date;
DEFINE INDEX idx_je_status ON journal_entry FIELDS status;

-- ────────────────────────────────────────────────────────────────
-- journal_line — سطور القيد (مدين / دائن)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE journal_line SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update, delete WHERE false;

DEFINE FIELD entry        ON journal_line TYPE record<journal_entry>;
DEFINE FIELD account      ON journal_line TYPE record<account_chart>;
DEFINE FIELD debit        ON journal_line TYPE float DEFAULT 0
    ASSERT $value >= 0;
DEFINE FIELD credit       ON journal_line TYPE float DEFAULT 0
    ASSERT $value >= 0;
DEFINE FIELD description  ON journal_line TYPE option<string>;
DEFINE FIELD cost_center  ON journal_line TYPE option<record<cost_center>>;
DEFINE FIELD created_at   ON journal_line TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_jl_entry   ON journal_line FIELDS entry;
DEFINE INDEX idx_jl_account ON journal_line FIELDS account;

-- ────────────────────────────────────────────────────────────────
-- vat_register — سجل ضريبة القيمة المضافة
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE vat_register SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD period_start   ON vat_register TYPE datetime;
DEFINE FIELD period_end     ON vat_register TYPE datetime;
DEFINE FIELD output_vat     ON vat_register TYPE option<float>;
DEFINE FIELD input_vat      ON vat_register TYPE option<float>;
DEFINE FIELD net_vat        ON vat_register TYPE option<float>;
DEFINE FIELD total_sales    ON vat_register TYPE option<float>;
DEFINE FIELD total_purchases ON vat_register TYPE option<float>;
DEFINE FIELD status         ON vat_register TYPE string
    ASSERT $value IN ["draft", "filed", "paid"]
    DEFAULT "draft";
DEFINE FIELD filing_date    ON vat_register TYPE option<datetime>;
DEFINE FIELD payment_date   ON vat_register TYPE option<datetime>;
DEFINE FIELD notes          ON vat_register TYPE option<string>;
DEFINE FIELD created_at     ON vat_register TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_vat_period ON vat_register FIELDS period_start, period_end;

-- ────────────────────────────────────────────────────────────────
-- ZATCA fields on invoice — حقول الفوترة الإلكترونية
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD uuid            ON invoice TYPE option<string>;
DEFINE FIELD seller_vat      ON invoice TYPE option<string>;
DEFINE FIELD buyer_vat       ON invoice TYPE option<string>;
DEFINE FIELD seller_name     ON invoice TYPE option<string>;
DEFINE FIELD seller_address  ON invoice TYPE option<string>;
DEFINE FIELD buyer_address   ON invoice TYPE option<string>;
DEFINE FIELD invoice_type    ON invoice TYPE string
    ASSERT $value IN ["standard", "simplified", "debit_note", "credit_note"]
    DEFAULT "standard";
DEFINE FIELD qr_code         ON invoice TYPE option<string>;
DEFINE FIELD hash            ON invoice TYPE option<string>;
DEFINE FIELD cost_center     ON invoice TYPE option<record<cost_center>>;

-- soft delete on new accounting tables
DEFINE FIELD is_archived ON account_chart   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON fiscal_period   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON cost_center     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON journal_entry   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON journal_line    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON vat_register    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON company_settings TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١١: الموارد البشرية المتقدمة (HR International)   ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- department — الهيكل التنظيمي (شجري)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE department SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD code        ON department TYPE string;
DEFINE FIELD name        ON department TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD parent      ON department TYPE option<record<department>>;
DEFINE FIELD manager     ON department TYPE option<record<account>>;
DEFINE FIELD cost_center ON department TYPE option<record<cost_center>>;
DEFINE FIELD is_active   ON department TYPE bool DEFAULT true;
DEFINE FIELD created_at  ON department TYPE datetime DEFAULT time::now();
DEFINE FIELD organization ON department TYPE option<record<organization>>;

DEFINE INDEX idx_dept_code ON department FIELDS code UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- position — الوظائف والدرجات (Job Catalog & Grading)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE position SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD code            ON position TYPE string;
DEFINE FIELD title           ON position TYPE string;
-- FIELD title_en removed (unified to title)
DEFINE FIELD department      ON position TYPE option<record<department>>;
DEFINE FIELD grade           ON position TYPE option<string>;
DEFINE FIELD min_salary      ON position TYPE option<float>;
DEFINE FIELD max_salary      ON position TYPE option<float>;
DEFINE FIELD description     ON position TYPE option<string>;
DEFINE FIELD requirements    ON position TYPE array DEFAULT [];
DEFINE FIELD is_active       ON position TYPE bool DEFAULT true;
DEFINE FIELD headcount       ON position TYPE int DEFAULT 1;
DEFINE FIELD created_at      ON position TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_pos_code ON position FIELDS code UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `employee` — إثراء جدول الموظف
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD department      ON employee TYPE option<record<department>>;
DEFINE FIELD position        ON employee TYPE option<record<position>>;
DEFINE FIELD employee_number ON employee TYPE option<string>;
DEFINE FIELD iqama_number    ON employee TYPE option<string>;
DEFINE FIELD iqama_expiry    ON employee TYPE option<string>;
DEFINE FIELD passport_number ON employee TYPE option<string>;
DEFINE FIELD passport_expiry ON employee TYPE option<string>;
DEFINE FIELD nationality     ON employee TYPE option<string>;
DEFINE FIELD religion        ON employee TYPE option<string>;
DEFINE FIELD marital_status  ON employee TYPE option<string>;
DEFINE FIELD dependents      ON employee TYPE int DEFAULT 0;
DEFINE FIELD bank_name       ON employee TYPE option<string>;
DEFINE FIELD bank_iban       ON employee TYPE option<string>;
DEFINE FIELD emergency_name  ON employee TYPE option<string>;
DEFINE FIELD emergency_phone ON employee TYPE option<string>;
DEFINE FIELD emergency_relation ON employee TYPE option<string>;
DEFINE FIELD base_salary     ON employee TYPE option<float>;
DEFINE FIELD housing_allowance ON employee TYPE option<float>;
DEFINE FIELD transport_allowance ON employee TYPE option<float>;
DEFINE FIELD employment_type ON employee TYPE string
    ASSERT $value IN ["full_time", "part_time", "contract", "intern", "probation"]
    DEFAULT "full_time";
DEFINE FIELD probation_end   ON employee TYPE option<string>;
DEFINE FIELD contract_end    ON employee TYPE option<string>;
DEFINE FIELD termination_date ON employee TYPE option<string>;
DEFINE FIELD termination_reason ON employee TYPE option<string>;
DEFINE FIELD photo_url       ON employee TYPE option<string>;

-- ────────────────────────────────────────────────────────────────
-- warning — الإنذارات والجزاءات (Disciplinary Actions)
-- مادة ٦٦–٦٨ من نظام العمل السعودي
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE warning SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD employee     ON warning TYPE record<employee>;
DEFINE FIELD issued_by    ON warning TYPE record<account>;
DEFINE FIELD warning_type ON warning TYPE string
    ASSERT $value IN ["verbal", "written_1", "written_2", "final", "deduction", "suspension", "termination"];
DEFINE FIELD violation    ON warning TYPE string
    ASSERT $value IN [
        "absence", "tardiness", "negligence", "misconduct",
        "insubordination", "safety_violation", "harassment",
        "theft", "fraud", "substance_abuse", "policy_violation", "other"
    ];
DEFINE FIELD description   ON warning TYPE string;
DEFINE FIELD deduction_amount ON warning TYPE option<float>;
DEFINE FIELD deduction_days   ON warning TYPE int DEFAULT 0;
DEFINE FIELD suspension_days  ON warning TYPE int DEFAULT 0;
DEFINE FIELD date          ON warning TYPE datetime;
DEFINE FIELD expiry_date   ON warning TYPE option<datetime>;
DEFINE FIELD employee_ack  ON warning TYPE bool DEFAULT false;
DEFINE FIELD ack_date      ON warning TYPE option<datetime>;
DEFINE FIELD notes         ON warning TYPE option<string>;
DEFINE FIELD attachments   ON warning TYPE array DEFAULT [];
DEFINE FIELD created_at    ON warning TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_warning_emp ON warning FIELDS employee, date;

-- ────────────────────────────────────────────────────────────────
-- end_of_service — مكافأة نهاية الخدمة
-- نظام العمل السعودي مادة ٨٤–٨٦
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE end_of_service SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD employee         ON end_of_service TYPE record<employee>;
DEFINE FIELD termination_type ON end_of_service TYPE string
    ASSERT $value IN ["resignation", "termination", "end_of_contract", "retirement", "death", "mutual_agreement"];
DEFINE FIELD hire_date        ON end_of_service TYPE datetime;
DEFINE FIELD end_date         ON end_of_service TYPE datetime;
DEFINE FIELD total_years      ON end_of_service TYPE float;
DEFINE FIELD last_salary      ON end_of_service TYPE float;
DEFINE FIELD half_month_years ON end_of_service TYPE option<float>;
DEFINE FIELD full_month_years ON end_of_service TYPE option<float>;
DEFINE FIELD gross_amount     ON end_of_service TYPE float;
DEFINE FIELD deductions       ON end_of_service TYPE option<float>;
DEFINE FIELD net_amount       ON end_of_service TYPE float;
DEFINE FIELD leave_balance_payout ON end_of_service TYPE option<float>;
DEFINE FIELD total_payout     ON end_of_service TYPE float;
DEFINE FIELD status           ON end_of_service TYPE string
    ASSERT $value IN ["calculated", "approved", "paid"]
    DEFAULT "calculated";
DEFINE FIELD approved_by      ON end_of_service TYPE option<record<account>>;
DEFINE FIELD payment_date     ON end_of_service TYPE option<datetime>;
DEFINE FIELD notes            ON end_of_service TYPE option<string>;
DEFINE FIELD created_at       ON end_of_service TYPE datetime DEFAULT time::now();

-- ────────────────────────────────────────────────────────────────
-- overtime_request — طلبات العمل الإضافي
-- مادة ١٠٧ — ١٥٠٪ من الراتب
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE overtime_request SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee     ON overtime_request TYPE record<employee>;
DEFINE FIELD date         ON overtime_request TYPE datetime;
DEFINE FIELD hours        ON overtime_request TYPE float ASSERT $value > 0;
DEFINE FIELD reason       ON overtime_request TYPE string;
DEFINE FIELD rate_multiplier ON overtime_request TYPE float DEFAULT 1.5;
DEFINE FIELD calculated_amount ON overtime_request TYPE option<float>;
DEFINE FIELD status       ON overtime_request TYPE string
    ASSERT $value IN ["pending", "approved", "rejected"]
    DEFAULT "pending";
DEFINE FIELD approved_by  ON overtime_request TYPE option<record<account>>;
DEFINE FIELD notes        ON overtime_request TYPE option<string>;
DEFINE FIELD created_at   ON overtime_request TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_overtime_emp_date ON overtime_request FIELDS employee, date;

-- ────────────────────────────────────────────────────────────────
-- training_program — برامج التدريب والتطوير
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE training_program SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD title         ON training_program TYPE string;
DEFINE FIELD description   ON training_program TYPE option<string>;
DEFINE FIELD provider      ON training_program TYPE option<string>;
DEFINE FIELD program_type  ON training_program TYPE string
    ASSERT $value IN ["internal", "external", "online", "on_the_job", "certification"]
    DEFAULT "internal";
DEFINE FIELD category      ON training_program TYPE string
    ASSERT $value IN ["technical", "safety", "management", "compliance", "soft_skills", "other"]
    DEFAULT "technical";
DEFINE FIELD start_date    ON training_program TYPE datetime;
DEFINE FIELD end_date      ON training_program TYPE option<datetime>;
DEFINE FIELD total_hours   ON training_program TYPE option<float>;
DEFINE FIELD cost          ON training_program TYPE option<float>;
DEFINE FIELD max_participants ON training_program TYPE int DEFAULT 20;
DEFINE FIELD location      ON training_program TYPE option<string>;
DEFINE FIELD instructor    ON training_program TYPE option<string>;
DEFINE FIELD status        ON training_program TYPE string
    ASSERT $value IN ["planned", "in_progress", "completed", "cancelled"]
    DEFAULT "planned";
DEFINE FIELD notes         ON training_program TYPE option<string>;
DEFINE FIELD created_at    ON training_program TYPE datetime DEFAULT time::now();
DEFINE FIELD organization  ON training_program TYPE option<record<organization>>;

-- ────────────────────────────────────────────────────────────────
-- training_enrollment — تسجيل الموظفين في البرامج
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE training_enrollment SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee      ON training_enrollment TYPE record<employee>;
DEFINE FIELD program       ON training_enrollment TYPE record<training_program>;
DEFINE FIELD enrollment_date ON training_enrollment TYPE datetime DEFAULT time::now();
DEFINE FIELD completion_date ON training_enrollment TYPE option<datetime>;
DEFINE FIELD score          ON training_enrollment TYPE option<float>;
DEFINE FIELD certificate_id ON training_enrollment TYPE option<record<certificate>>;
DEFINE FIELD status         ON training_enrollment TYPE string
    ASSERT $value IN ["enrolled", "in_progress", "completed", "dropped", "failed"]
    DEFAULT "enrolled";
DEFINE FIELD feedback       ON training_enrollment TYPE option<string>;
DEFINE FIELD created_at     ON training_enrollment TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_trnl_emp_prog ON training_enrollment FIELDS employee, program UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- employee_document — وثائق الموظف (إقامة، جواز، رخص)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE employee_document SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role = "admin"
        FOR delete WHERE false;

DEFINE FIELD employee       ON employee_document TYPE record<employee>;
DEFINE FIELD document_type  ON employee_document TYPE string
    ASSERT $value IN [
        "iqama", "passport", "national_id", "driving_license",
        "professional_license", "health_certificate", "insurance_card",
        "work_permit", "education_certificate", "contract", "other"
    ];
DEFINE FIELD document_number ON employee_document TYPE option<string>;
DEFINE FIELD issue_date      ON employee_document TYPE option<datetime>;
DEFINE FIELD expiry_date     ON employee_document TYPE option<datetime>;
DEFINE FIELD issuing_authority ON employee_document TYPE option<string>;
DEFINE FIELD file_url        ON employee_document TYPE option<string>;
DEFINE FIELD notes           ON employee_document TYPE option<string>;
DEFINE FIELD alert_days      ON employee_document TYPE int DEFAULT 30;
DEFINE FIELD created_at      ON employee_document TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_empdoc_emp ON employee_document FIELDS employee;
DEFINE INDEX idx_empdoc_expiry ON employee_document FIELDS expiry_date;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `leave_request` — أنواع إجازات إضافية
-- ────────────────────────────────────────────────────────────────
-- Redefine leave_type to include Saudi / international leave types

DEFINE FIELD attachment_url ON leave_request TYPE option<string>;
DEFINE FIELD return_date    ON leave_request TYPE option<datetime>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `leave_balance` — أرصدة إضافية
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD hajj_total       ON leave_balance TYPE int DEFAULT 15;
DEFINE FIELD hajj_used        ON leave_balance TYPE int DEFAULT 0;
DEFINE FIELD maternity_total  ON leave_balance TYPE int DEFAULT 70;
DEFINE FIELD maternity_used   ON leave_balance TYPE int DEFAULT 0;
DEFINE FIELD paternity_total  ON leave_balance TYPE int DEFAULT 3;
DEFINE FIELD paternity_used   ON leave_balance TYPE int DEFAULT 0;
DEFINE FIELD marriage_total   ON leave_balance TYPE int DEFAULT 5;
DEFINE FIELD marriage_used    ON leave_balance TYPE int DEFAULT 0;
DEFINE FIELD bereavement_total ON leave_balance TYPE int DEFAULT 5;
DEFINE FIELD bereavement_used ON leave_balance TYPE int DEFAULT 0;
DEFINE FIELD carry_over       ON leave_balance TYPE int DEFAULT 0;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `job_application` — خط أنابيب التوظيف
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD job_title       ON job_application TYPE option<string>;
DEFINE FIELD department      ON job_application TYPE option<record<department>>;
DEFINE FIELD resume_url      ON job_application TYPE option<string>;
DEFINE FIELD interview_date  ON job_application TYPE option<datetime>;
DEFINE FIELD interview_notes ON job_application TYPE option<string>;
DEFINE FIELD interviewer     ON job_application TYPE option<record<account>>;
DEFINE FIELD offered_salary  ON job_application TYPE option<float>;
DEFINE FIELD rejection_reason ON job_application TYPE option<string>;
DEFINE FIELD source          ON job_application TYPE option<string>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `attendance` — تحسينات الحضور
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD overtime_hours  ON attendance TYPE option<float>;
DEFINE FIELD late_minutes    ON attendance TYPE int DEFAULT 0;
DEFINE FIELD early_leave_min ON attendance TYPE int DEFAULT 0;
DEFINE FIELD shift           ON attendance TYPE option<string>;
DEFINE FIELD source          ON attendance TYPE string DEFAULT "manual";
DEFINE FIELD location        ON attendance TYPE option<string>;

-- soft delete on new HR tables
DEFINE FIELD is_archived ON department           TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON position             TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON warning              TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON end_of_service       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON overtime_request     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON training_program     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON training_enrollment  TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON employee_document    TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٢: CRM المتقدم (Advanced CRM)                    ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `client` — إثراء ملف العميل
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD client_type     ON client TYPE string
    ASSERT $value IN ["company", "individual", "government", "ngo"]
    DEFAULT "company";
DEFINE FIELD classification  ON client TYPE string
    ASSERT $value IN ["vip", "key", "regular", "prospect"]
    DEFAULT "regular";
DEFINE FIELD segment         ON client TYPE option<string>;
DEFINE FIELD source          ON client TYPE string
    ASSERT $value IN ["referral", "website", "cold_call", "exhibition", "social_media", "advertising", "partner", "other"]
    DEFAULT "other";
DEFINE FIELD website         ON client TYPE option<string>;
DEFINE FIELD vat_number      ON client TYPE option<string>;
DEFINE FIELD cr_number       ON client TYPE option<string>;
DEFINE FIELD payment_terms   ON client TYPE string
    ASSERT $value IN ["immediate", "net_15", "net_30", "net_45", "net_60", "net_90"]
    DEFAULT "net_30";
DEFINE FIELD account_manager ON client TYPE option<record<account>>;
DEFINE FIELD rating          ON client TYPE int
    ASSERT $value >= 0 AND $value <= 5
    DEFAULT 0;
DEFINE FIELD total_revenue   ON client TYPE option<float>;
DEFINE FIELD outstanding_balance ON client TYPE option<float>;
DEFINE FIELD last_interaction ON client TYPE option<datetime>;
DEFINE FIELD country         ON client TYPE option<string>;
DEFINE FIELD district        ON client TYPE option<string>;
DEFINE FIELD postal_code     ON client TYPE option<string>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `contact` — إثراء جهة الاتصال
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD department      ON contact TYPE option<string>;
DEFINE FIELD mobile          ON contact TYPE option<string>;
DEFINE FIELD linkedin        ON contact TYPE option<string>;
DEFINE FIELD preferred_lang  ON contact TYPE string DEFAULT "ar";
DEFINE FIELD gender          ON contact TYPE option<string>;
DEFINE FIELD birth_date      ON contact TYPE option<datetime>;
DEFINE FIELD last_contacted  ON contact TYPE option<datetime>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `interaction` — إثراء سجل التفاعلات
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD duration_min    ON interaction TYPE int DEFAULT 0;
DEFINE FIELD outcome         ON interaction TYPE string
    ASSERT $value IN ["positive", "neutral", "negative", "no_response", "pending"]
    DEFAULT "pending";
DEFINE FIELD opportunity     ON interaction TYPE option<record<opportunity>>;
DEFINE FIELD priority        ON interaction TYPE string
    ASSERT $value IN ["low", "medium", "high", "urgent"]
    DEFAULT "medium";
DEFINE FIELD is_completed    ON interaction TYPE bool DEFAULT false;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `opportunity` — إثراء فرص البيع
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD source          ON opportunity TYPE string
    ASSERT $value IN ["referral", "website", "cold_call", "exhibition", "existing_client", "partner", "other"]
    DEFAULT "other";
DEFINE FIELD competitor      ON opportunity TYPE option<string>;
DEFINE FIELD weighted_value  ON opportunity TYPE option<float>;
DEFINE FIELD next_action     ON opportunity TYPE option<string>;
DEFINE FIELD next_action_date ON opportunity TYPE option<datetime>;
DEFINE FIELD products        ON opportunity TYPE array DEFAULT [];
DEFINE FIELD won_date        ON opportunity TYPE option<datetime>;
DEFINE FIELD lost_date       ON opportunity TYPE option<datetime>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `quotation` — إثراء عروض الأسعار
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD discount_rate   ON quotation TYPE option<float>;
DEFINE FIELD discount_amount ON quotation TYPE option<float>;
DEFINE FIELD delivery_terms  ON quotation TYPE option<string>;
DEFINE FIELD payment_terms   ON quotation TYPE string
    ASSERT $value IN ["immediate", "net_15", "net_30", "net_45", "net_60"]
    DEFAULT "net_30";
DEFINE FIELD prepared_by     ON quotation TYPE option<record<account>>;
DEFINE FIELD version         ON quotation TYPE int DEFAULT 1;
DEFINE FIELD converted_to    ON quotation TYPE option<record<invoice>>;

-- ────────────────────────────────────────────────────────────────
-- client_tag — تصنيفات العملاء (Tags)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE client_tag SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role = "admin";

DEFINE FIELD name       ON client_tag TYPE string;
DEFINE FIELD color      ON client_tag TYPE string DEFAULT "#3B82F6";
DEFINE FIELD created_at ON client_tag TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_tag_name ON client_tag FIELDS name UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- client_tag_link — ربط العميل بالتصنيفات (M-to-M)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE client_tag_link SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD client     ON client_tag_link TYPE record<client>;
DEFINE FIELD tag        ON client_tag_link TYPE record<client_tag>;
DEFINE FIELD created_at ON client_tag_link TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_ctl_unique ON client_tag_link FIELDS client, tag UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- client_feedback — ملاحظات ورضا العملاء (NPS)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE client_feedback SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client       ON client_feedback TYPE record<client>;
DEFINE FIELD contact      ON client_feedback TYPE option<record<contact>>;
DEFINE FIELD feedback_type ON client_feedback TYPE string
    ASSERT $value IN ["nps", "csat", "complaint", "suggestion", "compliment"]
    DEFAULT "csat";
DEFINE FIELD score        ON client_feedback TYPE option<int>;
DEFINE FIELD category     ON client_feedback TYPE option<string>;
DEFINE FIELD description  ON client_feedback TYPE string;
DEFINE FIELD resolution   ON client_feedback TYPE option<string>;
DEFINE FIELD status       ON client_feedback TYPE string
    ASSERT $value IN ["open", "in_progress", "resolved", "closed"]
    DEFAULT "open";
DEFINE FIELD priority     ON client_feedback TYPE string
    ASSERT $value IN ["low", "medium", "high", "critical"]
    DEFAULT "medium";
DEFINE FIELD assigned_to  ON client_feedback TYPE option<record<account>>;
DEFINE FIELD resolved_at  ON client_feedback TYPE option<datetime>;
DEFINE FIELD created_at   ON client_feedback TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_feedback_client ON client_feedback FIELDS client;
DEFINE INDEX idx_feedback_status ON client_feedback FIELDS status;

-- ────────────────────────────────────────────────────────────────
-- sales_target — أهداف المبيعات
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE sales_target SCHEMAFULL
    PERMISSIONS
        FOR select WHERE employee = $auth.id OR $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD employee     ON sales_target TYPE option<record<account>>;
DEFINE FIELD department   ON sales_target TYPE option<record<department>>;
DEFINE FIELD period_type  ON sales_target TYPE string
    ASSERT $value IN ["monthly", "quarterly", "annual"]
    DEFAULT "monthly";
DEFINE FIELD year         ON sales_target TYPE int;
DEFINE FIELD period       ON sales_target TYPE int;
DEFINE FIELD target_amount ON sales_target TYPE float;
DEFINE FIELD achieved_amount ON sales_target TYPE option<float>;
DEFINE FIELD target_deals ON sales_target TYPE int DEFAULT 0;
DEFINE FIELD achieved_deals ON sales_target TYPE int DEFAULT 0;
DEFINE FIELD currency     ON sales_target TYPE string DEFAULT "SAR";
DEFINE FIELD notes        ON sales_target TYPE option<string>;
DEFINE FIELD created_at   ON sales_target TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_target_period ON sales_target FIELDS employee, year, period UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- service_agreement — عقود الخدمة / SLA
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE service_agreement SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD client         ON service_agreement TYPE record<client>;
DEFINE FIELD agreement_number ON service_agreement TYPE string;
DEFINE FIELD title          ON service_agreement TYPE string;
DEFINE FIELD description    ON service_agreement TYPE option<string>;
DEFINE FIELD agreement_type ON service_agreement TYPE string
    ASSERT $value IN ["maintenance", "support", "managed_service", "warranty", "consulting"]
    DEFAULT "maintenance";
DEFINE FIELD start_date     ON service_agreement TYPE datetime;
DEFINE FIELD end_date       ON service_agreement TYPE datetime;
DEFINE FIELD value          ON service_agreement TYPE option<float>;
DEFINE FIELD billing_cycle  ON service_agreement TYPE string
    ASSERT $value IN ["monthly", "quarterly", "semi_annual", "annual", "one_time"]
    DEFAULT "monthly";
DEFINE FIELD response_time_hours ON service_agreement TYPE int DEFAULT 24;
DEFINE FIELD resolution_time_hours ON service_agreement TYPE int DEFAULT 72;
DEFINE FIELD max_visits     ON service_agreement TYPE option<int>;
DEFINE FIELD visits_used    ON service_agreement TYPE int DEFAULT 0;
DEFINE FIELD auto_renew     ON service_agreement TYPE bool DEFAULT false;
DEFINE FIELD status         ON service_agreement TYPE string
    ASSERT $value IN ["draft", "active", "expired", "cancelled", "renewed"]
    DEFAULT "draft";
DEFINE FIELD notes          ON service_agreement TYPE option<string>;
DEFINE FIELD created_at     ON service_agreement TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_sa_number ON service_agreement FIELDS agreement_number UNIQUE;
DEFINE INDEX idx_sa_client ON service_agreement FIELDS client;

-- soft delete on new CRM tables
DEFINE FIELD is_archived ON client_tag         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON client_tag_link    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON client_feedback    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON sales_target       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON service_agreement  TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٣: كتالوج المنتجات والخدمات (Product & Service)  ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- product_category — تصنيفات المنتجات (شجري)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE product_category SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD code       ON product_category TYPE string;
DEFINE FIELD name       ON product_category TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD parent     ON product_category TYPE option<record<product_category>>;
DEFINE FIELD image_url  ON product_category TYPE option<string>;
DEFINE FIELD is_active  ON product_category TYPE bool DEFAULT true;
DEFINE FIELD created_at ON product_category TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_pcat_code ON product_category FIELDS code UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- product — المنتجات (آلات، معدات، قطع غيار)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE product SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD sku            ON product TYPE string;
DEFINE FIELD name           ON product TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD category       ON product TYPE option<record<product_category>>;
DEFINE FIELD product_type   ON product TYPE string
    ASSERT $value IN ["machine", "spare_part", "accessory", "consumable", "tool", "other"]
    DEFAULT "machine";
DEFINE FIELD brand          ON product TYPE option<string>;
DEFINE FIELD model          ON product TYPE option<string>;
DEFINE FIELD description    ON product TYPE option<string>;
DEFINE FIELD specifications ON product TYPE object DEFAULT {};
DEFINE FIELD unit           ON product TYPE string DEFAULT "piece";
DEFINE FIELD sell_price     ON product TYPE option<float>;
DEFINE FIELD cost_price     ON product TYPE option<float>;
DEFINE FIELD currency       ON product TYPE string DEFAULT "SAR";
DEFINE FIELD tax_rate       ON product TYPE option<float>;
DEFINE FIELD weight_kg      ON product TYPE option<float>;
DEFINE FIELD dimensions     ON product TYPE option<string>;
DEFINE FIELD image_url      ON product TYPE option<string>;
DEFINE FIELD images         ON product TYPE array DEFAULT [];
DEFINE FIELD datasheet_url  ON product TYPE option<string>;
DEFINE FIELD vendor         ON product TYPE option<record<vendor>>;
DEFINE FIELD warranty_months ON product TYPE int DEFAULT 12;
DEFINE FIELD min_stock      ON product TYPE int DEFAULT 0;
DEFINE FIELD is_active      ON product TYPE bool DEFAULT true;
DEFINE FIELD is_sellable    ON product TYPE bool DEFAULT true;
DEFINE FIELD is_purchasable ON product TYPE bool DEFAULT true;
DEFINE FIELD notes          ON product TYPE option<string>;
DEFINE FIELD created_at     ON product TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_product_sku ON product FIELDS sku UNIQUE;
DEFINE INDEX idx_product_cat ON product FIELDS category;

-- ────────────────────────────────────────────────────────────────
-- service_catalog — كتالوج الخدمات
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE service_catalog SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD code            ON service_catalog TYPE string;
DEFINE FIELD name            ON service_catalog TYPE string;
-- FIELD name_en removed (unified to name)
DEFINE FIELD category        ON service_catalog TYPE string
    ASSERT $value IN [
        "installation", "repair", "maintenance", "calibration",
        "inspection", "training", "consulting", "support", "other"
    ]
    DEFAULT "repair";
DEFINE FIELD description     ON service_catalog TYPE option<string>;
DEFINE FIELD base_price      ON service_catalog TYPE option<float>;
DEFINE FIELD price_type      ON service_catalog TYPE string
    ASSERT $value IN ["fixed", "hourly", "per_visit", "custom"]
    DEFAULT "fixed";
DEFINE FIELD estimated_hours ON service_catalog TYPE option<float>;
DEFINE FIELD currency        ON service_catalog TYPE string DEFAULT "SAR";
DEFINE FIELD tax_rate        ON service_catalog TYPE option<float>;
DEFINE FIELD required_skills ON service_catalog TYPE array DEFAULT [];
DEFINE FIELD required_parts  ON service_catalog TYPE array DEFAULT [];
DEFINE FIELD is_active       ON service_catalog TYPE bool DEFAULT true;
DEFINE FIELD notes           ON service_catalog TYPE option<string>;
DEFINE FIELD created_at      ON service_catalog TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_svc_code ON service_catalog FIELDS code UNIQUE;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٤: إثراء الصيانة والآلات (Service Enrichments)   ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `machine` — ربط الآلات بالأقسام الأخرى
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD client        ON machine TYPE option<record<client>>;
DEFINE FIELD product       ON machine TYPE option<record<product>>;
DEFINE FIELD vendor        ON machine TYPE option<record<vendor>>;
DEFINE FIELD department    ON machine TYPE option<record<department>>;
DEFINE FIELD assigned_tech ON machine TYPE option<record<employee>>;
DEFINE FIELD installation_date ON machine TYPE option<datetime>;
DEFINE FIELD last_service  ON machine TYPE option<datetime>;
DEFINE FIELD next_service  ON machine TYPE option<datetime>;
DEFINE FIELD operating_hours ON machine TYPE option<float>;
DEFINE FIELD photo_url     ON machine TYPE option<string>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `repair_operation` — علاقات مباشرة
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD machine       ON repair_operation TYPE option<record<machine>>;
DEFINE FIELD client        ON repair_operation TYPE option<record<client>>;
DEFINE FIELD project       ON repair_operation TYPE option<record<project>>;
DEFINE FIELD technician    ON repair_operation TYPE option<record<employee>>;
DEFINE FIELD service       ON repair_operation TYPE option<record<service_catalog>>;
DEFINE FIELD parts_list    ON repair_operation TYPE array DEFAULT [];
DEFINE FIELD labor_cost    ON repair_operation TYPE option<float>;
DEFINE FIELD parts_cost    ON repair_operation TYPE option<float>;
DEFINE FIELD total_cost    ON repair_operation TYPE option<float>;
DEFINE FIELD invoice       ON repair_operation TYPE option<record<invoice>>;
DEFINE FIELD warranty      ON repair_operation TYPE option<record<warranty_card>>;
DEFINE FIELD priority      ON repair_operation TYPE string
    ASSERT $value IN ["low", "medium", "high", "critical"]
    DEFAULT "medium";
DEFINE FIELD satisfaction  ON repair_operation TYPE option<int>;
DEFINE FIELD photos        ON repair_operation TYPE array DEFAULT [];

-- ────────────────────────────────────────────────────────────────
-- service_visit — زيارات الصيانة الميدانية
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE service_visit SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR delete WHERE false;

DEFINE FIELD client          ON service_visit TYPE record<client>;
DEFINE FIELD machine         ON service_visit TYPE option<record<machine>>;
DEFINE FIELD technician      ON service_visit TYPE record<employee>;
DEFINE FIELD service         ON service_visit TYPE option<record<service_catalog>>;
DEFINE FIELD service_contract ON service_visit TYPE option<record<service_contract>>;
DEFINE FIELD repair          ON service_visit TYPE option<record<repair_operation>>;
DEFINE FIELD visit_date      ON service_visit TYPE datetime;
DEFINE FIELD arrival_time    ON service_visit TYPE option<datetime>;
DEFINE FIELD departure_time  ON service_visit TYPE option<datetime>;
DEFINE FIELD visit_type      ON service_visit TYPE string
    ASSERT $value IN ["scheduled", "emergency", "follow_up", "installation", "inspection"]
    DEFAULT "scheduled";
DEFINE FIELD description     ON service_visit TYPE string;
DEFINE FIELD work_performed  ON service_visit TYPE option<string>;
DEFINE FIELD parts_used      ON service_visit TYPE array DEFAULT [];
DEFINE FIELD client_signature ON service_visit TYPE option<string>;
DEFINE FIELD photos          ON service_visit TYPE array DEFAULT [];
DEFINE FIELD status          ON service_visit TYPE string
    ASSERT $value IN ["planned", "in_progress", "completed", "cancelled"]
    DEFAULT "planned";
DEFINE FIELD notes           ON service_visit TYPE option<string>;
DEFINE FIELD created_at      ON service_visit TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_sv_client ON service_visit FIELDS client;
DEFINE INDEX idx_sv_date   ON service_visit FIELDS visit_date;
DEFINE INDEX idx_sv_tech   ON service_visit FIELDS technician;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `warranty_card` — ربط ببطاقة المنتج
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD client   ON warranty_card TYPE option<record<client>>;
DEFINE FIELD machine  ON warranty_card TYPE option<record<machine>>;
DEFINE FIELD product  ON warranty_card TYPE option<record<product>>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `service_contract` — ربط بالمنتج
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD assigned_tech ON service_contract TYPE option<record<employee>>;
DEFINE FIELD department    ON service_contract TYPE option<record<department>>;
DEFINE FIELD visits_log    ON service_contract TYPE array DEFAULT [];

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `inventory_item` — ربط بالمنتج والمستودع
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD product    ON inventory_item TYPE option<record<product>>;
DEFINE FIELD warehouse  ON inventory_item TYPE option<record<warehouse>>;
DEFINE FIELD vendor     ON inventory_item TYPE option<record<vendor>>;
DEFINE FIELD barcode    ON inventory_item TYPE option<string>;
DEFINE FIELD image_url  ON inventory_item TYPE option<string>;
DEFINE FIELD expiry_date ON inventory_item TYPE option<datetime>;
DEFINE FIELD last_restock ON inventory_item TYPE option<datetime>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `purchase_order` — ربط بالمشروع
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD project     ON purchase_order TYPE option<record<project>>;
DEFINE FIELD department  ON purchase_order TYPE option<record<department>>;
DEFINE FIELD received_at ON purchase_order TYPE option<datetime>;
DEFINE FIELD invoice     ON purchase_order TYPE option<record<invoice>>;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `expense` — ربط بالمشروع
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD department  ON expense TYPE option<record<department>>;

-- soft delete on all new tables
DEFINE FIELD is_archived ON project_member      TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON board               TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON board_list          TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card               TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_member        TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_label         TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_label_link    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_checklist     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON checklist_item     TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_comment       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_attachment    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON card_activity      TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON product_category   TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON product            TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON service_catalog    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON service_visit      TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء ١٥: قائمة المواد والتصنيع (BOM & Manufacturing)  ██
-- ████████████████████████████████████████████████████████████████

-- ────────────────────────────────────────────────────────────────
-- bom — قائمة المواد (Bill of Materials)
-- كل منتج (ماكينة) يملك BOM يحدد مكوناته
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE bom SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD product       ON bom TYPE record<product>;
DEFINE FIELD bom_number    ON bom TYPE string;
DEFINE FIELD title         ON bom TYPE string;
DEFINE FIELD version       ON bom TYPE int DEFAULT 1;
DEFINE FIELD description   ON bom TYPE option<string>;
DEFINE FIELD total_cost    ON bom TYPE option<float>;
DEFINE FIELD labor_cost    ON bom TYPE option<float>;
DEFINE FIELD overhead_cost ON bom TYPE option<float>;
DEFINE FIELD yield_qty     ON bom TYPE int DEFAULT 1;
DEFINE FIELD unit          ON bom TYPE string DEFAULT "piece";
DEFINE FIELD status        ON bom TYPE string
    ASSERT $value IN ["draft", "active", "obsolete"]
    DEFAULT "draft";
DEFINE FIELD approved_by   ON bom TYPE option<record<account>>;
DEFINE FIELD approved_at   ON bom TYPE option<datetime>;
DEFINE FIELD notes         ON bom TYPE option<string>;
DEFINE FIELD created_at    ON bom TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at    ON bom TYPE datetime VALUE time::now();

DEFINE INDEX idx_bom_number ON bom FIELDS bom_number UNIQUE;
DEFINE INDEX idx_bom_product ON bom FIELDS product;

-- ────────────────────────────────────────────────────────────────
-- bom_line — سطر مكوّن في قائمة المواد
-- مثال: ماكينة طباعة ← (1× لوح PCB, 3× مقاومة 10K, 1× متحكم STM32)
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE bom_line SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE $auth.role IN ["admin", "manager"];

DEFINE FIELD bom              ON bom_line TYPE record<bom>;
DEFINE FIELD line_number      ON bom_line TYPE int;
DEFINE FIELD component        ON bom_line TYPE option<record<product>>;
DEFINE FIELD inventory_item   ON bom_line TYPE option<record<inventory_item>>;
DEFINE FIELD description      ON bom_line TYPE string;
DEFINE FIELD quantity         ON bom_line TYPE float ASSERT $value > 0;
DEFINE FIELD unit             ON bom_line TYPE string DEFAULT "piece";
DEFINE FIELD unit_cost        ON bom_line TYPE option<float>;
DEFINE FIELD total_cost       ON bom_line TYPE option<float>;
DEFINE FIELD waste_percentage ON bom_line TYPE option<float>;
DEFINE FIELD is_optional      ON bom_line TYPE bool DEFAULT false;
DEFINE FIELD substitute       ON bom_line TYPE option<record<product>>;
DEFINE FIELD notes            ON bom_line TYPE option<string>;
DEFINE FIELD created_at       ON bom_line TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_bomline_bom ON bom_line FIELDS bom, line_number UNIQUE;

-- ────────────────────────────────────────────────────────────────
-- production_order — أمر إنتاج / تجميع
-- عند طلب ماكينة → يُنشأ أمر إنتاج → يخصم المكونات تلقائياً
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE production_order SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD order_number    ON production_order TYPE string;
DEFINE FIELD product         ON production_order TYPE record<product>;
DEFINE FIELD bom             ON production_order TYPE record<bom>;
DEFINE FIELD quantity         ON production_order TYPE int ASSERT $value > 0;
DEFINE FIELD client          ON production_order TYPE option<record<client>>;
DEFINE FIELD project         ON production_order TYPE option<record<project>>;
DEFINE FIELD department      ON production_order TYPE option<record<department>>;
DEFINE FIELD assigned_to     ON production_order TYPE option<record<employee>>;
DEFINE FIELD planned_start   ON production_order TYPE option<datetime>;
DEFINE FIELD planned_end     ON production_order TYPE option<datetime>;
DEFINE FIELD actual_start    ON production_order TYPE option<datetime>;
DEFINE FIELD actual_end      ON production_order TYPE option<datetime>;
DEFINE FIELD estimated_cost  ON production_order TYPE option<float>;
DEFINE FIELD actual_cost     ON production_order TYPE option<float>;
DEFINE FIELD priority        ON production_order TYPE string
    ASSERT $value IN ["low", "medium", "high", "urgent"]
    DEFAULT "medium";
DEFINE FIELD status          ON production_order TYPE string
    ASSERT $value IN [
        "draft", "confirmed", "material_ready", "in_production",
        "quality_check", "completed", "cancelled"
    ]
    DEFAULT "draft";
DEFINE FIELD quality_notes   ON production_order TYPE option<string>;
DEFINE FIELD serial_numbers  ON production_order TYPE array DEFAULT [];
DEFINE FIELD notes           ON production_order TYPE option<string>;
DEFINE FIELD created_by      ON production_order TYPE option<record<account>>;
DEFINE FIELD created_at      ON production_order TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at      ON production_order TYPE datetime VALUE time::now();

DEFINE INDEX idx_po_number ON production_order FIELDS order_number UNIQUE;
DEFINE INDEX idx_po_status ON production_order FIELDS status;

-- ────────────────────────────────────────────────────────────────
-- production_order_line — استهلاك المواد الفعلي
-- يتتبع: الكمية المطلوبة vs الكمية المستهلكة فعلاً
-- يربط بـ inventory_transaction لخصم المخزون
-- ────────────────────────────────────────────────────────────────
DEFINE TABLE production_order_line SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee"]
        FOR create, update WHERE $auth.role IN ["admin", "manager"]
        FOR delete WHERE false;

DEFINE FIELD production_order ON production_order_line TYPE record<production_order>;
DEFINE FIELD bom_line         ON production_order_line TYPE option<record<bom_line>>;
DEFINE FIELD inventory_item   ON production_order_line TYPE record<inventory_item>;
DEFINE FIELD description      ON production_order_line TYPE string;
DEFINE FIELD planned_qty      ON production_order_line TYPE float ASSERT $value > 0;
DEFINE FIELD actual_qty       ON production_order_line TYPE option<float>;
DEFINE FIELD unit             ON production_order_line TYPE string DEFAULT "piece";
DEFINE FIELD unit_cost        ON production_order_line TYPE option<float>;
DEFINE FIELD total_cost       ON production_order_line TYPE option<float>;
DEFINE FIELD consumed_at      ON production_order_line TYPE option<datetime>;
DEFINE FIELD inv_transaction  ON production_order_line TYPE option<record<inventory_transaction>>;
DEFINE FIELD status           ON production_order_line TYPE string
    ASSERT $value IN ["pending", "reserved", "consumed", "returned"]
    DEFAULT "pending";
DEFINE FIELD notes            ON production_order_line TYPE option<string>;
DEFINE FIELD created_at       ON production_order_line TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_pol_order ON production_order_line FIELDS production_order;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `product` — ربط المنتج بقائمة المواد
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD active_bom   ON product TYPE option<record<bom>>;
DEFINE FIELD has_bom      ON product TYPE bool DEFAULT false;

-- ────────────────────────────────────────────────────────────────
-- Enrichments on `inventory_transaction` — ربط بأمر الإنتاج
-- ────────────────────────────────────────────────────────────────
DEFINE FIELD production_order ON inventory_transaction TYPE option<record<production_order>>;

-- soft delete on BOM tables
DEFINE FIELD is_archived ON bom                    TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON bom_line               TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON production_order       TYPE bool DEFAULT false;
DEFINE FIELD is_archived ON production_order_line  TYPE bool DEFAULT false;

-- ████████████████████████████████████████████████████████████████
-- ██  الجزء N: الفيديوهات التعليمية (Educational Videos)      ██
-- ████████████████████████████████████████████████████████████████

-- educational_video
DEFINE TABLE educational_video SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.role IN ["admin", "manager", "employee", "intern"]
        FOR create, update, delete WHERE $auth.role = "admin";

DEFINE FIELD title           ON educational_video TYPE string
    ASSERT string::len($value) > 0;
DEFINE FIELD description     ON educational_video TYPE option<string>;
DEFINE FIELD file_path       ON educational_video TYPE string;
DEFINE FIELD file_name       ON educational_video TYPE string;
DEFINE FIELD file_size       ON educational_video TYPE int    DEFAULT 0;
DEFINE FIELD mime_type       ON educational_video TYPE string DEFAULT "video/mp4";
DEFINE FIELD duration_secs   ON educational_video TYPE option<int>;
DEFINE FIELD uploaded_by     ON educational_video TYPE record<account>;
DEFINE FIELD created_at      ON educational_video TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_video_title   ON educational_video FIELDS title;
DEFINE INDEX idx_video_created ON educational_video FIELDS created_at;
