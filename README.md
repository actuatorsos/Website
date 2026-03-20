# Actuators | البنية التحتية الرقمية للاقتصاد الريادي

<div align="center">

![Rust](https://img.shields.io/badge/Rust-Nightly-orange?logo=rust&style=for-the-badge)
![Axum](https://img.shields.io/badge/Axum-0.8-blue?style=for-the-badge)
![SurrealDB](https://img.shields.io/badge/SurrealDB-2.x-purple?style=for-the-badge)
![HTMX](https://img.shields.io/badge/HTMX-1.9-blue?style=for-the-badge)
![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-3.x-38B2AC?logo=tailwind-css&logoColor=white&style=for-the-badge)

**مؤسسة رقمية ونظام تشغيل اقتصادي خفيف الوزن، يعمل كعمود فقري لتوحيد وتوجيه منظومة ريادة الأعمال المجزأة.**

</div>

---

## 📋 نظرة عامة (Overview)

منصة **Actuators** هي نظام إدارة وبنية تحتية لامركزية مبنية لربط الشركات الناشئة والفرق التطوعية بشبكات المستثمرين ورأس مال الشتات عبر حوض البحر المتوسط. يهدف النظام إلى حل مشاكل تفتت الاقتصاد والبيئة الهشة، وخلق بيئة "ثقة رقمية" استثمارية تدمج بين ثلاثة محاور رئيسية:
1. **نظام إدارة متكامل (ERP):** إدارة الموارد، المهام، العمليات والمستندات بأسلوب مؤسسي.
2. **بوابة الاستثمار والشتات:** ربط المؤسسين بالمستثمرين والمرشدين دولياً لدعم وتدفق رأس المال.
3. **مسارات تعليمية (LMS):** دمج مسارات التعلم لسد الفجوة المعرفية بالتوازي مع التنفيذ.

### ⚙️ التقنيات المستخدمة (Tech Stack)
- **الخادم (Backend):** مبني بلغة **Rust** من خلال بيئة العمل **Axum** لتوفير خوادم سريعة وملتزمة بأمان الذاكرة.
- **قاعدة البيانات (Database):** **SurrealDB** كقاعدة بيانات NoSQL مع قدرات Graph لترابط البيانات (شركات، مستثمرين، مسارات تعليمية).
- **الواجهة (Frontend):** معالجة قوالب SSR فائقة السرعة باستخدام **Askama**، ومحسنة تفاعلياً بـ **HTMX** بدون الحاجة لأطر JavaScript ضخمة، مع واجهات هندسية منسقة بـ **TailwindCSS** توافق قواعد هوية Actuators الصارمة.

---

## 🚀 دليل التشغيل السريع (Quick Start)

لتشغيل بنية النظام بنجاح محلياً، تأكد من اتباع الخطوات التالية:

### 1️⃣ المتطلبات المسبقة (Prerequisites)
- [Rust](https://rustup.rs/) (Nightly / Stable) محدث.
- [SurrealDB](https://surrealdb.com/install) للبيانات المحلية.

### 2️⃣ تشغيل قاعدة البيانات (SurrealDB Engine)
افتح نافذة موجه أوامر (Terminal) الخاصة بالنظام الأساسي ونفذ:
```powershell
surreal start --user Actuators --pass Actuators123 --bind 0.0.0.0:8000 file:Actuators.db
```
> ⚠️ **تنويه:** يجب أن تبقى محطة البيانات قيد التشغيل لربط المعاملات الخاصة بالـ ERP.

### 3️⃣ تشغيل الخادم الرئيسي (Core Server)
في نافذة (Terminal) أخرى في نفس المسار، قم بتهيئة واجهة التشغيل:
```powershell
cargo run --bin dr_machine_web
```
> سيظهر لك إبروتوكول البث: `🚀 Server starting at http://0.0.0.0:3000`

---

## 🧭 محاور النظام (Entry Nodes)

- 🏠 **الصفحة الرئيسية (الواجهة التعريفية):** [http://localhost:3000](http://localhost:3000)
- 🏢 **البنية والهندسة (من نحن):** [http://localhost:3000/about](http://localhost:3000/about)
- 🎯 **لوحة تحكم النظام (Admin/ERP):** [http://localhost:3000/admin](http://localhost:3000/admin)

---

## 🔐 نظام الحوكمة والوصول (Access Control)

لدخول وحدة التحكم الإدارية المركزية `/admin`، ستحتاج لبيانات المشرف المبدئية:
- **معرف النظام (User):** `actuators.os@gmail.com`
- **مفتاح العبور (Password):** `Actuators_2024`

*(يعتمد النظام على JWT Tokens لحوكمة الوصول وحماية المسارات الاستثمارية والبيانات الحساسة).*

---

## 📁 الهيكل المنهجي للمشروع (System Architecture)

تم تقسيم البنية لضمان التوسع والمرونة المؤسسية (Modular Monolith):

```text
dr_machine_web/
├── src/                      # النواة (Rust Source Code)
│   ├── main.rs               # نقطة التهيئة، وخادم Axum
│   ├── build.rs              # مترجم إعدادات النظام
│   ├── domains/              # النطاقات التشغيلية
│   │   ├── auth/             # بوابة الحوكمة والمصادقة
│   │   ├── users/            # إدارة الفرق، المؤسسين، والمستثمرين
│   │   ├── machinery/        # رقمنة الأصول الاستثمارية والعمليات
│   │   ├── projects_adv/     # لوحة تتبع مسارات الـ LMS والمشاريع
│   │   └── documents/        # نظام إدارة المعرفة والمستندات المؤسسية
├── templates/                # قوالب العرض (Askama + HTMX)
│   ├── admin/                # واجهات الـ ERP للمستوى الإداري
│   ├── fragments/            # المكونات الديناميكية الدقيقة (UI Widgets)
│   └── ...                   # بنية الواجهة العامة (Isometric Theme)
├── static/                   # ملفات الأنماط الخطية (JetBrains, IBM Plex)
├── locales/                  # ترجمات النظام متعدد اللغات (AR, EN)
├── dr_machine.db/            # محرك قاعدة بيانات SurrealDB المحلي
├── .env                      # إعدادات التهيئة للبيئة
└── Cargo.toml                # تبعيات ومكتبات النظام الأساسية
```

---

## 🛠️ تتبع النظام وتصحيح الأخطاء (System Diagnostics)

| حالة النظام                                    | الحل                                                                                        |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------- |
| **انقطاع تدفق البيانات (لا يوجد اتصال بـ DB)** | تأكد من أن منفذ `surreal start` مفتوح، وأن `SURREAL_URL` في ملف `.env` هو `127.0.0.1:8000`. |
| **تم رفض الوصول (404 / 401)**                  | تأكد من صلاحية الـ Cookies أو مفاتيح الـ Auth. النظام يتطلب تحقق قوي للجلسة.                |
| **التأثيرات البصرية لا تُحدّث (UI Desync)**      | قم بتحديث كاش المتصفح (`Ctrl + F5`) لإعادة تحميل إطارات TailwindCSS وهيكل الإيزومتريك.      |

---

<div align="center">
  <span style="font-family: 'JetBrains Mono', monospace; font-size: 0.85rem;">END OF DOCUMENT // ACTUATORS DIGITAL INFRASTRUCTURE</span><br>
  <b>جميع الحقوق محفوظة .</b><br>
  <i>يمنع نشر أو استخدام هذا التكوين خارج نطاق بيئة الاختبار دون التصاريح المؤسسية.</i>
</div>
