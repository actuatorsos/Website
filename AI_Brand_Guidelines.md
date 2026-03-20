# Actuators: AI UI/UX Brand Guidelines (System Prompt)

هذا الملف مخصص كدليل تعليمات شامل (System Prompt / Context) لنماذج الذكاء الاصطناعي (AI/LLMs) عند توليد الأكواد (HTML/CSS/JS, React, Slint, Rust) أو عند كتابة المحتوى لمشروع **Actuators**. يجب على الذكاء الاصطناعي الالتزام الصارم بهذه القواعد لضمان اتساق الهوية البصرية والهندسية للمشروع.

---

## 1. الهوية المؤسسية والنبرة (Brand Essence & Tone)
- **الشخصية (Persona):** ثقة مؤسسية قوية (Institutional trust)، قوة هادئة، منطق هندسي ونظامي (Clean geometric logic).
- **المحاذير (Anti-patterns):** مشروع Actuators ليس "شركة ناشئة مرحة" ولا "منظمة غير حكومية تقليدية". يُمنع استخدام الألوان الزاهية المتعددة الفوضوية، الصور النمطية للإنسان والسعادة المجتمعية، أو الأنماط المبالغ فيها (Cyberpunk cliches).
- **لغة التصميم (Design Language):** هندسة بصرية إيزومترية (Isometric)، رسوم بيانية للشبكات (Node maps)، وواجهات تقنية صارمة وحادة.

---

## 2. نظام الألوان المزدوج (Dual-Theme Color System)
التصميم الأساسي يعتمد على لوحة ألوان ثلاثية عالية التباين. على الذكاء الاصطناعي دائمًا تضمين متغيرات `CSS / Theme Variables` لدعم الوضعين الداكن والفاتح، مع الانتباه لتغير دور كل منهما بناءً على الهوية البصرية المُحدَّثة.

### النمط الفاتح (Light Mode - Default Presentation)
النمط الأساسي الافتراضي لعروض التقديم، الواجهات العامة، والمحتوى المكتوب:
- `Surface / Background` : `#F8FAFC` أو الأبيض النقي (يعكس النقاء المؤسسي).
- `Surface Highlight` : `#FFFFFF` (خلفيات البطاقات مع حدود واضحة).
- `Text Primary` : `#0B1A2A` (كحلي غامق كخط أساسي).
- `Text Muted` : `#475569` (رمادي داكن).
- `Structure / Lines` : `#00B7C2` (Cyan - يُستخدم للخطوط، الحدود، التقسيمات، والأيقونات الهيكلية).
- `Activation / Signal` : `#FF6A2A` (Orange - يُستخدم للنقاط النشطة، الأزرار الرئيسية، وحالة النظام المكتملة فقط).

### النمط الداكن (Dark Mode - For UI & Dashboards)
مُخصص فقط لواجهات التحكم (Dashboards)، اللوحات التقنية (UI Panels)، والتخطيطات التي تتطلب تبايناً عالياً:
- `Surface / Background` : `#0B1A2A` (كحلي غامق).
- `Surface Highlight` : `rgba(11, 26, 42, 0.8)` (خلفيات البطاقات).
- `Text Primary` : `#FFFFFF` (أبيض قاطع).
- `Text Muted` : `rgba(255, 255, 255, 0.6)`.
- `Structure / Lines` : `#00B7C2` (Cyan).
- `Activation / Signal` : `#FF6A2A` (Orange).

---

## 3. الخطوط واللغات (Typography & Bilingual Support)
يجب على الأكواد المولدة دعم الاتجاهين (LTR للإنجليزية، و RTL للعربية) بمرونة عالية، مع استخدام الخطوط المناسبة لكل لغة للتشابه في الطابع المؤسسي.

### الإنجليزية (English Typography)
- **Primary Font:** `Inter` (للعناوين والنصوص الأساسية. محايد، حديث، واحترافي).
- **Secondary/Mono Font:** `JetBrains Mono` (للمقاييس، معرفات النظام Node_IDs، التسميات التوضيحية البنيوية).

### العربية (Arabic Typography)
- **Primary Arabic Font:** `IBM Plex Sans Arabic` أو `janna lt` (للحفاظ على الطابع الهندسي والمؤسسي الصارم المتوافق مع Inter).
- **البيانات والأرقام:** في الجداول الإحصائية أو حالات النظام، يُفضل إبقاء الأرقام (1,2,3) والرموز التقنية بخط `JetBrains Mono` حتى ضمن الواجهات العربية (Monospaced output).
- **توجيه التصميم (RTL):** تأكد دائماً في كود CSS من استخدام الخصائص المنطقية مثل `margin-inline-start` بدلاً من `margin-left` لدعم الانعكاس السلس (Bi-directional UI).

---

## 4. مكونات واجهة المستخدم والقواعد العامة (UI Components Rules)
1. **الحدود الهيكلية (Borders & Grids):** يعتمد التصميم على الخطوط الرفيعة شبه الشفافة `1px` لرسم شبكات الواجهة. في الدارك مود `border: 1px solid rgba(0, 183, 194, 0.3)`. لا يوجد فواصل مخفية، بل واضحة وصريحة.
2. **الزوايا (Border Radius):** زوايا حادة تماماً (`0px`) أو بانحناء طفيف جداً لا يتجاوز (`2px` - `4px`) كحد أقصى للحفاظ على الشكل المؤسسي. لا تستخدم الزوايا الدائرية الكبيرة (Pill-shaped buttons).
3. **تكوين الرسوميات (Illustration Logic):** عند تصميم رسومات مساعدة، يجب بناؤها من مكعبات، عقد (nodes)، ومقاطع خطية (line segments). **يُمنع استخدام الأشكال المملوءة الثقيلة (Heavy filled shapes) أو الأشكال الكرتونية الناعمة (Soft cartoon icons)**. يعتمد التصميم على البنية السلكية المفتوحة (Open wireframe language).
4. **نظام الشعار (Logo System):** الشعار يتكون من 7 مكعبات إيزومترية باللون السيان (Cyan) يتوسطها مكعب برتقالي (Orange core) نشط. يجب الحفاظ على مساحات بيضاء واسعة حول الشعار (Generous white space).
5. **التأثيرات والظلال (Shadows & Effects):**
   - **الظلال (Drop-shadows):** ممنوعة للزينة. لا تستخدم ظلالاً ناعمة وكبيرة.
   - **تأثير الزجاج (Glassmorphism):** ممنوع بشكل مبالغ فيه.
   - **التدرجات (Gradients):** مسموحة فقط للإشارة بشكل وظيفي للانتقال الحركي أو تنشيط النظام (Activation Logic)، وليست كخلفيات رئيسية للبطاقات. الفلات كلر (Flat color) هو الأساس.
6. **خلفيات النظام:** يجب تزيين خلفيات الحاوية الرئيسية بشبكة هندسية خفيفة (Grid background)، يمكن تحقيقها باستخدام CSS `linear-gradient` لتمثيل التقاطعات الإيزومترية.

---

## 5. قواعد الحركة وتفاعل المستخدم (Motion Principles)
- **غياب التمدد (No Elastic Easing):** يجب تجنب تأثيرات الارتداد (Bouncing/Elastic). حركات العناصر يجب أن تبدو ميكانيكية، دقيقة، وقاطعة.
- **التجميع المتسلسل (Sequential Assembly):** عند تحميل واجهة، تظهر المكونات بترتيب تسلسلي كما لو كانت أجزاء من شبكة تترابط مع بعضها (Snapping into interlocking grids).
- **نبض التفعيل (Activation Pulse):** اللون البرتقالي يمكن استخدامه كمؤشر حالة تفعيل (Pulse effect) عندما تكون هناك بيانات جديدة أو عملية ناجحة.

---

## 6. أوجه التنفيذ التقنية (Strict CSS Variables Example)
عند كتابة أي كود جديد، ابدأ بتأسيس النظام اللوني كالتالي لتسهيل التبديل بين اللغات والأنماط (Themes):

```css
:root {
  /* Light Theme (Default Presentation) */
  --bg-primary: #F8FAFC;
  --bg-surface: #FFFFFF;
  --text-primary: #0B1A2A;
  --text-muted: #475569;
  --structure-cyan: #00B7C2;
  --structure-cyan-muted: rgba(0, 183, 194, 0.3);
  --activation-orange: #FF6A2A;
  --font-primary-en: 'Inter', sans-serif;
  --font-primary-ar: 'IBM Plex Sans Arabic', 'Tajawal', sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
}

[data-theme="dark"] {
  /* Dark Theme (For UI Panels & Dashboards) */
  --bg-primary: #0B1A2A;
  --bg-surface: rgba(11, 26, 42, 0.8);
  --text-primary: #FFFFFF;
  --text-muted: rgba(255, 255, 255, 0.6);
  --structure-cyan: #00B7C2;
  --structure-cyan-muted: rgba(0, 183, 194, 0.3);
  --activation-orange: #FF6A2A;
}

/* Bi-directional support */
[dir="rtl"] {
  font-family: var(--font-primary-ar);
}
[dir="ltr"] {
  font-family: var(--font-primary-en);
}
```

```
> [!IMPORTANT]
> أيها المبرمج / الذكاء الاصطناعي: اقرأ هذا الدليل جيداً قبل كتابة سطر كود واحد خاص بمشروع Actuators. الدقة الهندسية والابتعاد عن العشوائية أو الزخرفة الزائدة هي جوهر هذا النظام التشغيلي الاقتصادي.
```
