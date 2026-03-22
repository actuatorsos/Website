use std::collections::HashMap;
use std::sync::LazyLock;

/// Static translation table: key -> (English, Arabic).
static TRANSLATIONS: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(translations);

/// Build the translation map.
///
/// Each entry is `key => (en, ar)`.
fn translations() -> HashMap<&'static str, (&'static str, &'static str)> {
    HashMap::from([
        // ── Navigation ─────────────────────────────────────────
        ("dashboard",   ("Dashboard",    "لوحة التحكم")),
        ("employees",   ("Employees",    "الموظفون")),
        ("departments", ("Departments",  "الأقسام")),
        ("settings",    ("Settings",     "الإعدادات")),
        ("audit",       ("Audit Log",    "سجل التدقيق")),
        ("login",       ("Login",        "تسجيل الدخول")),
        ("register",    ("Register",     "إنشاء حساب")),
        ("logout",      ("Logout",       "تسجيل الخروج")),
        ("profile",     ("Profile",      "الملف الشخصي")),

        // ── Form labels ────────────────────────────────────────
        ("email",    ("Email",    "البريد الإلكتروني")),
        ("password", ("Password", "كلمة المرور")),
        ("name",     ("Name",     "الاسم")),
        ("phone",    ("Phone",    "الهاتف")),
        ("save",     ("Save",     "حفظ")),
        ("cancel",   ("Cancel",   "إلغاء")),
        ("create",   ("Create",   "إنشاء")),
        ("edit",     ("Edit",     "تعديل")),
        ("delete",   ("Delete",   "حذف")),
        ("search",   ("Search",   "بحث")),
        ("filter",   ("Filter",   "تصفية")),

        // ── Messages ───────────────────────────────────────────
        ("welcome",      ("Welcome",       "مرحباً")),
        ("not_found",    ("Not Found",     "غير موجود")),
        ("unauthorized", ("Unauthorized",  "غير مصرح")),
        ("forbidden",    ("Forbidden",     "محظور")),
        ("success",      ("Success",       "نجاح")),
        ("error",        ("Error",         "خطأ")),
        ("loading",      ("Loading…",      "جارٍ التحميل…")),

        // ── HR terms ───────────────────────────────────────────
        ("employee",   ("Employee",    "موظف")),
        ("department", ("Department",  "قسم")),
        ("job_title",  ("Job Title",   "المسمى الوظيفي")),
        ("contract",   ("Contract",    "عقد")),
        ("hire_date",  ("Hire Date",   "تاريخ التعيين")),
        ("status",     ("Status",      "الحالة")),
    ])
}

/// Look up a translation by `key` for the given `lang` (`"en"` or `"ar"`).
///
/// Returns the English text when the language is unrecognised, or an empty
/// string if the key itself is missing from the table.
pub fn t(key: &str, lang: &str) -> &'static str {
    match TRANSLATIONS.get(key) {
        Some(&(en, ar)) => match lang {
            "ar" => ar,
            _ => en,
        },
        None => "",
    }
}
