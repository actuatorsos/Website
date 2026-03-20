-- ============================================================================
-- DUMMY DATA FOR TESTING — بيانات تجريبية للاختبار
-- ============================================================================

-- 1. Store Settings
UPDATE store_settings:main SET
    store_name = 'متجر د. مكينة للأصناف الصناعية',
    store_-- name_en = 'Dr. Machine Industrial Store',
    description = 'المتجر الرسمي لقطع الغيار والمعدات الصناعية والحلول التقنية',
    currency = 'SAR',
    tax_rate = 15.0,
    shipping_flat = 50.0,
    free_shipping_above = 1000.0,
    is_active = true,
    contact_email = 'store@dr-machine.com',
    contact_phone = '+966500000000';

-- 2. Sample Products (if any missing)
UPSERT product:p1 SET sku = 'MCH-001', name = 'ماكينة CNC ليزر 40 واط', sell_price = 4500.0, is_active = true, is_sellable = true;
UPSERT product:p2 SET sku = 'SP-088', name = 'طقم فلاتر زيت هيدروليك', sell_price = 120.0, is_active = true, is_sellable = true;
UPSERT product:p3 SET sku = 'ELC-99', name = 'متحكم منطكي مبرمج PLC Delta', sell_price = 850.0, is_active = true, is_sellable = true;

-- 3. Sample Services
UPSERT service_catalog:s1 SET code = 'INST-01', name = 'تركيب وبرمجة ماكينة CNC', base_price = 500.0, is_active = true;
UPSERT service_catalog:s2 SET code = 'MAIN-12', name = 'صيانة دورية وقائية (زيارة واحدة)', base_price = 350.0, is_active = true;

-- 4. Sample Clients
UPSERT client:c1 SET company_name = 'شركة الصناعات المتقدمة', contact_person = 'خالد محمد', email = 'khaled@adv-ind.com', status = 'active';
UPSERT client:c2 SET company_name = 'مصنع السلام للأثاث', contact_person = 'فهد العتيبي', email = 'fahad@salam-factory.com', status = 'active';

-- 5. Sample Store Orders
UPSERT store_order:ord1 SET 
    order_number = 'ORD-2026-0001',
    customer = client:c1,
    customer_name = 'خالد محمد',
    status = 'processing',
    subtotal = 4620.0,
    tax_amount = 693.0,
    shipping_cost = 0.0,
    total = 5313.0,
    created_at = time::now() - 2d;

UPSERT order_item:oi1 SET order = store_order:ord1, item_type = 'product', product = product:p1, name = 'ماكينة CNC ليزر 40 واط', quantity = 1, unit_price = 4500.0, total_price = 4500.0;
UPSERT order_item:oi2 SET order = store_order:ord1, item_type = 'product', product = product:p2, name = 'طقم فلاتر زيت هيدروليك', quantity = 1, unit_price = 120.0, total_price = 120.0;

UPSERT store_order:ord2 SET 
    order_number = 'ORD-2026-0002',
    customer = client:c2,
    customer_name = 'فهد العتيبي',
    status = 'pending',
    subtotal = 350.0,
    tax_amount = 52.5,
    shipping_cost = 50.0,
    total = 452.5,
    created_at = time::now() - 5h;

UPSERT order_item:oi3 SET order = store_order:ord2, item_type = 'service', service = service_catalog:s2, name = 'صيانة دورية وقائية (زيارة واحدة)', quantity = 1, unit_price = 350.0, total_price = 350.0;

-- 6. AI Agents & Policies
-- Note: api_key_hash is a dummy hash for "test-key"
UPSERT agent_account:bot1 SET 
    name = 'وكيل الدعم الفني', 
    description = 'وكيل آلي للرد على استفسارات الصيانة والقطع',
    api_key_hash = '$argon2id$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$P5/Q7/V8yYw', 
    scopes = ['hr.read', 'store.read', 'email.send'],
    rate_limit = 60;

UPSERT action_policy:p1 SET 
    action = 'salary.update', 
    scope_required = 'hr.salary.edit', 
    sensitivity = 'critical', 
    requires_human = true;

UPSERT action_policy:p2 SET 
    action = 'inventory.adjust', 
    scope_required = 'inventory.write', 
    sensitivity = 'sensitive', 
    requires_human = true;

UPSERT action_policy:p3 SET 
    action = 'leave.approve', 
    scope_required = 'hr.leave.approve', 
    sensitivity = 'routine', 
    requires_human = false;

UPSERT approval_request:req1 SET 
    agent = agent_account:bot1,
    action = 'salary.update',
    target_table = 'salary',
    payload = { employee: 'account:emp123', new_salary: 15000 },
    reason = 'بناءً على تقييم الأداء الربعي المرتفع للموظف',
    status = 'pending',
    priority = 'high';

-- 7. Email Configuration & Templates
UPSERT email_config:main SET 
    host = 'smtp.sendgrid.net',
    port = 587,
    username = 'apikey',
    password = 'SG.fake_password_for_testing',
    from_email = 'no-reply@dr-machine.com',
    from_name = 'نظام د. مكينة',
    use_tls = true,
    is_active = true;

UPSERT email_template:welcome SET 
    name = 'ترحيب بالموظف الجديد',
    subject = 'مرحباً بك في فريق د. مكينة',
    body = '<h1>أهلاً بك {{name}}!</h1><p>يسعدنا انضمامك لفريقنا التقني...</p>',
    category = 'welcome',
    variables = ['name'];

UPSERT email_log:l1 SET 
    recipient = 'test@example.com',
    subject = 'تنبيه تسجيل دخول جديد',
    status = 'sent',
    sent_at = time::now() - 1h;

-- 8. Notifications
-- Note: Assuming account:admin exists
UPSERT notification:n1 SET 
    recipient = account:admin,
    title = 'طلب موافقة جديد',
    message = 'الوكيل السريع يطلب تعديل راتب الموظف "أحمد علي"',
    notif_type = 'warning',
    created_at = time::now() - 10m;

UPSERT notification:n2 SET 
    recipient = account:admin,
    title = 'اكتمال جرد المخزون',
    message = 'تمت عملية الجرد الشهري للمستودع الرئيسي بنجاح',
    notif_type = 'success',
    is_read = true,
    created_at = time::now() - 2h;
