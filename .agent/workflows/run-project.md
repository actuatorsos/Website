---
description: طريقة تشغيل مشروع Actuators
---

# تشغيل مشروع Actuators

## المتطلبات
- Rust (Nightly أو Stable حديث)
- SurrealDB

---

## الخطوات

### 1. تشغيل قاعدة البيانات
افتح Terminal جديد في مجلد المشروع:

// turbo
```powershell
surreal start --user root --pass root123 --bind 0.0.0.0:8000 file:dr_machine.db
```

> اترك هذه النافذة مفتوحة

---

### 2. تشغيل التطبيق
افتح Terminal آخر في مجلد المشروع:

// turbo
```powershell
cargo run
```

> انتظر ظهور: `🚀 Server starting at http://0.0.0.0:3000`

---

### 3. فتح المتصفح
- الصفحة الرئيسية: http://localhost:3000
- من نحن: http://localhost:3000/about
- لوحة التحكم: http://localhost:3000/admin

---

## دخول لوحة التحكم

عند الدخول إلى `/admin`، ستظهر نافذة تسجيل دخول:

| الحقل            | القيمة  |
| ---------------- | ------- |
| **اسم المستخدم** | `admin` |
| **كلمة المرور**  | ``      |

---

## استكشاف الأخطاء

**إذا لم يتصل التطبيق بقاعدة البيانات:**
- تأكد من أن SurrealDB يعمل في نافذة منفصلة
- تأكد من أن ملف `.env` يحتوي على: `SURREAL_URL=127.0.0.1:8000`

**إذا ظهرت أخطاء ترجمة:**
- تأكد من وجود مجلد `locales/` يحتوي على `ar.json` و `en.json`