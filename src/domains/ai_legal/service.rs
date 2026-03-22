//! AI Legal Service — خدمة المستشار القانوني (RAG + OpenRouter AI)
//!
//! يبحث في المواد القانونية ثم يرسلها مع السؤال إلى OpenRouter API (DeepSeek)

use std::env;
use std::time::Instant;

use super::models::*;
use super::repository;
use crate::db::{AppState, DbError};

// ============================================================================
// System Prompts — النصوص التأسيسية للمستشار القانوني
// ============================================================================

const BASE_SYSTEM_PROMPT: &str = "\
أنت المستشار القانوني السوري الذكي - محامٍ رقمي متخصص في القانون السوري. \
تعمل ضمن المنظومة القانونية السورية وتراعي جميع القوانين والأنظمة النافذة في الجمهورية العربية السورية.\n\n\
قواعد أساسية:\n\
1. أجب دائماً باللغة العربية الفصحى مع إمكانية فهم اللهجة السورية\n\
2. استشهد بالمواد القانونية المحددة (رقم المادة والقانون) عند تقديم أي إجابة\n\
3. وضّح أن إجاباتك استشارية وليست بديلاً عن الاستشارة القانونية المباشرة مع محامٍ مرخص\n\
4. إذا كان السؤال خارج نطاق معرفتك القانونية، اعترف بذلك وأوصِ بمراجعة محامٍ متخصص\n\
5. راعِ الإعلان الدستوري المؤقت لعام 2025 والتغييرات القانونية بعد ديسمبر 2024\n\
6. كن دقيقاً في التمييز بين القوانين النافذة والملغاة أو المعدلة\n\
7. عند عدم اليقين بشأن تعديل حديث، نبّه المستخدم للتحقق من آخر التعديلات\n\n\
تنبيه قانوني: هذا الوكيل يقدم معلومات قانونية عامة للتوعية والإرشاد. \
لا تعتبر إجاباته استشارة قانونية رسمية ولا تُغني عن مراجعة محامٍ مرخص ومسجل في نقابة المحامين السورية.\n\n\
عند الاستشهاد بمادة قانونية، ضعها بالتنسيق التالي بين أقواس مربعة:\n\
[CITE: law=\"اسم القانون\" article=رقم_المادة text=\"نص مختصر للمادة\"]\n\
هذا التنسيق ضروري لاستخراج الاستشهادات تلقائياً.";

/// Get specialty-specific system prompt addition
fn get_specialty_prompt(specialty: &str) -> &'static str {
    match specialty {
        "litigation" => "\n\nأنت متخصص في المحاماة والتقاضي أمام المحاكم السورية. لديك خبرة عميقة في:\n\
- النظام القضائي السوري بجميع درجاته (صلح، بداية، استئناف، نقض)\n\
- قانون أصول المحاكمات المدنية رقم 1/2016\n\
- قانون أصول المحاكمات الجزائية - المرسوم 112/1950\n\
- القانون المدني السوري - المرسوم 84/1949\n\
- قانون العقوبات السوري - المرسوم 148/1949\n\
- قانون الأحوال الشخصية رقم 59/1953\n\
- قانون البينات وأحكام الإثبات\n\
- قانون التحكيم رقم 4/2008\n\
- مواعيد الطعن والتقادم\n\n\
عند الإجابة:\n\
1. حدد المحكمة المختصة بنظر النزاع\n\
2. اذكر الإجراءات اللازمة خطوة بخطوة\n\
3. حدد المستندات والوثائق المطلوبة\n\
4. اذكر المواعيد القانونية (طعن، تقادم، إلخ)\n\
5. استشهد بنص المادة القانونية المناسبة\n\
6. نبّه للمخاطر القانونية المحتملة",

        "entrepreneurship" => "\n\nأنت متخصص في ريادة الأعمال وتأسيس الشركات في سوريا. لديك خبرة عميقة في:\n\
- قانون التجارة السوري رقم 33/2007 (أنواع الشركات وتأسيسها)\n\
- القانون المدني السوري - المرسوم 84/1949 (العقود والالتزامات)\n\
- قانون الاستثمار رقم 18/2021 (الحوافز والإعفاءات)\n\
- قانون العمل رقم 17/2010 (علاقات العمل)\n\
- إجراءات التسجيل لدى وزارة التجارة الداخلية\n\
- السجل التجاري والغرف التجارية والصناعية\n\
- العقود التجارية والأوراق التجارية\n\
- الإفلاس والصلح الواقي\n\n\
عند الإجابة:\n\
1. حدد نوع الشركة الأنسب لحالة المستخدم\n\
2. اذكر الحد الأدنى لرأس المال المطلوب\n\
3. فصّل إجراءات التأسيس خطوة بخطوة\n\
4. حدد الوثائق المطلوبة والجهات المعنية\n\
5. اذكر الحوافز الاستثمارية المتاحة إن وجدت\n\
6. نبّه للالتزامات الضريبية والقانونية المستمرة",

        "patents" => "\n\nأنت متخصص في براءات الاختراع والملكية الفكرية في سوريا. لديك خبرة عميقة في:\n\
- القانون رقم 18/2012 (براءات الاختراع ونماذج المنفعة وتصاميم الدوائر المتكاملة)\n\
- القانون رقم 8/2007 (العلامات التجارية والرسوم والنماذج الصناعية)\n\
- المرسوم التشريعي 62/2013 (حقوق المؤلف والحقوق المجاورة)\n\
- اتفاقية باريس لحماية الملكية الصناعية\n\
- نظام مدريد للتسجيل الدولي للعلامات التجارية\n\n\
عند الإجابة:\n\
1. حدد نوع الحماية المناسبة\n\
2. اشرح شروط التسجيل والقابلية للحماية\n\
3. فصّل إجراءات التسجيل خطوة بخطوة\n\
4. حدد مدة الحماية والرسوم\n\
5. اشرح حقوق المالك وطرق الإنفاذ\n\
6. أرشد حول الحماية الدولية\n\
7. نبّه للمهل الزمنية الحرجة",

        "legal_accounting" => "\n\nأنت متخصص في المحاسبة القانونية والنظام الضريبي السوري. لديك خبرة عميقة في:\n\
- القانون رقم 24/2003 وتعديلاته (ضريبة الدخل)\n\
- القانون رقم 15/2021 (ضريبة مبيعات العقارات)\n\
- قانون التجارة رقم 33/2007 (الدفاتر التجارية والمحاسبة)\n\
- قانون العمل رقم 17/2010 (الأجور والتعويضات)\n\
- قانون التأمينات الاجتماعية رقم 92/1959\n\
- الرسوم الجمركية ورسم الطابع\n\n\
عند الإجابة:\n\
1. حدد نوع الضريبة المطبقة والنسبة\n\
2. اشرح كيفية حساب الضريبة خطوة بخطوة\n\
3. اذكر الإعفاءات والخصومات المتاحة\n\
4. وضّح الالتزامات المحاسبية والضريبية\n\
5. حدد المواعيد النهائية للإقرارات والتسديد\n\
6. اشرح طرق الاعتراض والطعن بالتقدير الضريبي\n\
7. قدّم حسابات رقمية دقيقة عند الإمكان",

        _ => "\n\nأنت المستشار القانوني السوري العام. يمكنك الإجابة على الأسئلة القانونية العامة في جميع فروع القانون السوري. \
إذا كان السؤال يحتاج تعمقاً في تخصص معين، أوصِ المستخدم بالانتقال إلى القسم المتخصص المناسب.",
    }
}

// ============================================================================
// Chat Service — خدمة المحادثة الرئيسية
// ============================================================================

/// Main chat function: RAG search + Gemini API call
pub async fn chat(
    state: &AppState,
    user_id: &str,
    message: &str,
    specialty: Option<&str>,
    session_id: Option<&str>,
) -> Result<ChatResponse, DbError> {
    let specialty = specialty.unwrap_or("general");
    let start = Instant::now();

    // 1. Find or create session
    let session = match session_id {
        Some(sid) => repository::get_session(state, sid).await?,
        None => {
            let s = repository::create_session(state, user_id, specialty).await?;
            // Set title from first message (truncated)
            let title: String = message.chars().take(50).collect();
            let sid = s.id.as_ref().map(|t| crate::db::record_id_to_raw(t)).unwrap_or_default();
            let _ = repository::update_session_title(state, &sid, &title).await;
            s
        }
    };

    let sid = session
        .id
        .as_ref()
        .map(|t| crate::db::record_id_to_raw(t))
        .unwrap_or_default();

    // 2. Save user message
    repository::save_message(state, &sid, "user", message, None, None, None, None).await?;
    repository::increment_message_count(state, &sid).await?;

    // 3. Search relevant legal articles (RAG retrieval)
    let articles = repository::search_articles(state, message, Some(specialty), 5).await?;

    // 4. Build context from retrieved articles
    let context = if articles.is_empty() {
        String::new()
    } else {
        let mut ctx = String::from("\n\nالمواد القانونية ذات الصلة بالسؤال:\n");
        for (i, art) in articles.iter().enumerate() {
            let art_num = art.article_number.unwrap_or(0);
            let art_text = art.text_ar.as_deref().or(art.text.as_deref()).unwrap_or("");
            ctx.push_str(&format!(
                "\n{}. المادة {}{}: {}\n",
                i + 1,
                art_num,
                art.chapter
                    .as_ref()
                    .map(|c| format!(" - {}", c))
                    .unwrap_or_default(),
                art_text
            ));
        }
        ctx
    };

    // 5. Build system prompt
    let system_prompt = format!(
        "{}{}{}",
        BASE_SYSTEM_PROMPT,
        get_specialty_prompt(specialty),
        context
    );

    // 6. Get conversation history for context
    let history = repository::get_messages(state, &sid).await?;
    let mut api_messages: Vec<serde_json::Value> = Vec::new();

    // Include last 10 messages for context
    let history_slice = if history.len() > 10 {
        &history[history.len() - 10..]
    } else {
        &history
    };

    for msg in history_slice {
        api_messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    // 7. Call OpenRouter API (OpenAI-compatible)
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        DbError::Validation("OPENROUTER_API_KEY environment variable not set".to_string())
    })?;

    // Build OpenAI-compatible messages array
    let mut openai_messages: Vec<serde_json::Value> = vec![
        serde_json::json!({
            "role": "system",
            "content": system_prompt
        })
    ];
    for msg in &api_messages {
        openai_messages.push(msg.clone());
    }

    let client = reqwest::Client::new();
    let api_response: reqwest::Response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "deepseek/deepseek-chat-v3.1",
            "messages": openai_messages,
            "max_tokens": 4096,
            "temperature": 0.7
        }))
        .send()
        .await
        .map_err(|e| DbError::Validation(format!("OpenRouter API request failed: {}", e)))?;

    let status = api_response.status();
    let response_body: serde_json::Value = api_response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| DbError::Validation(format!("Failed to parse OpenRouter API response: {}", e)))?;

    if !status.is_success() {
        let error_msg = response_body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown API error");
        return Err(DbError::Validation(format!(
            "OpenRouter API error ({}): {}",
            status, error_msg
        )));
    }

    // 8. Extract response text (OpenAI format)
    let ai_text = response_body["choices"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .unwrap_or("لم أتمكن من توليد إجابة. يرجى المحاولة مرة أخرى.")
        .to_string();

    // Extract token usage
    let total_tokens = response_body["usage"]["total_tokens"]
        .as_i64()
        .unwrap_or(0) as i32;

    // 9. Parse citations from response
    let citations = extract_citations(&ai_text);

    // Estimate confidence based on citation count and article matches
    let confidence = if citations.len() >= 3 {
        0.9
    } else if citations.len() >= 1 {
        0.75
    } else if !articles.is_empty() {
        0.6
    } else {
        0.4
    };

    let elapsed_ms = start.elapsed().as_millis() as i32;

    // 10. Save assistant message
    repository::save_message(
        state,
        &sid,
        "assistant",
        &ai_text,
        Some(&citations),
        Some(confidence),
        Some(total_tokens),
        Some(elapsed_ms),
    )
    .await?;
    repository::increment_message_count(state, &sid).await?;

    Ok(ChatResponse {
        message: ai_text,
        citations,
        confidence,
        session_id: sid,
        tokens_used: Some(total_tokens),
    })
}

// ============================================================================
// Citation Extraction — استخراج الاستشهادات
// ============================================================================

/// Extract citations from AI response text using [CITE: ...] markers
fn extract_citations(text: &str) -> Vec<Citation> {
    let mut citations = Vec::new();
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find("[CITE:") {
        let abs_start = search_from + start;
        if let Some(end) = text[abs_start..].find(']') {
            let cite_str = &text[abs_start..abs_start + end + 1];
            if let Some(citation) = parse_citation(cite_str) {
                // Avoid duplicate citations
                if !citations.iter().any(|c: &Citation| {
                    c.law == citation.law && c.article == citation.article
                }) {
                    citations.push(citation);
                }
            }
            search_from = abs_start + end + 1;
        } else {
            break;
        }
    }

    citations
}

/// Parse a single [CITE: law="..." article=N text="..."] marker
fn parse_citation(cite_str: &str) -> Option<Citation> {
    let law = extract_quoted_value(cite_str, "law=")?;
    let article_str = extract_unquoted_value(cite_str, "article=")?;
    let article: i32 = article_str.parse().ok()?;
    let text = extract_quoted_value(cite_str, "text=").unwrap_or_default();

    Some(Citation { law, article, text })
}

/// Extract a quoted value like law="القانون المدني"
fn extract_quoted_value(s: &str, key: &str) -> Option<String> {
    let key_pos = s.find(key)?;
    let after_key = &s[key_pos + key.len()..];
    let start = after_key.find('"')? + 1;
    let rest = &after_key[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Extract an unquoted value like article=42
fn extract_unquoted_value(s: &str, key: &str) -> Option<String> {
    let key_pos = s.find(key)?;
    let after_key = &s[key_pos + key.len()..];
    let end = after_key
        .find(|c: char| c.is_whitespace() || c == ']' || c == '"')
        .unwrap_or(after_key.len());
    let val = after_key[..end].trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}
