-- ============================================================
-- Actuators Platform — Cloud Seed Data
-- 3 Organizations: Actuators (platform), TechFlow (startup), Athar (volunteer)
-- ============================================================

-- ── Organizations ──
CREATE organization:actuators SET name = 'Actuators', plan = 'enterprise', industry = 'platform', owner = account:h3edv8uuruw4mxgosk43, created_at = time::now();
CREATE organization:techflow SET name = 'TechFlow Solutions', plan = 'startup', industry = 'technology', owner = account:h3edv8uuruw4mxgosk43, created_at = time::now();
CREATE organization:athar SET name = 'فريق أثر التطوعي', plan = 'free', industry = 'volunteer', owner = account:h3edv8uuruw4mxgosk43, created_at = time::now();

-- ── Link existing admin to Actuators org ──
UPDATE account:h3edv8uuruw4mxgosk43 SET organization = organization:actuators, role = 'admin', full_name = 'يحيى سوان';

-- ── TechFlow Accounts (startup) ──
CREATE account:tf_admin SET email = 'reem@techflow.sy', password_hash = crypto::bcrypt::generate('Test1234'), role = 'admin', organization = organization:techflow, full_name = 'ريم الخطيب', user_type = 'startup_owner', created_at = time::now();
CREATE account:tf_dev1 SET email = 'khalid@techflow.sy', password_hash = crypto::bcrypt::generate('Test1234'), role = 'employee', organization = organization:techflow, full_name = 'خالد المصري', user_type = 'member', created_at = time::now();
CREATE account:tf_dev2 SET email = 'lina@techflow.sy', password_hash = crypto::bcrypt::generate('Test1234'), role = 'employee', organization = organization:techflow, full_name = 'لينا حداد', user_type = 'member', created_at = time::now();
CREATE account:tf_sales SET email = 'omar@techflow.sy', password_hash = crypto::bcrypt::generate('Test1234'), role = 'employee', organization = organization:techflow, full_name = 'عمر النابلسي', user_type = 'member', created_at = time::now();

-- ── Athar Accounts (volunteer team) ──
CREATE account:at_admin SET email = 'leen@athar-team.org', password_hash = crypto::bcrypt::generate('Test1234'), role = 'admin', organization = organization:athar, full_name = 'لين العلي', user_type = 'volunteer', created_at = time::now();
CREATE account:at_vol1 SET email = 'bassel@athar-team.org', password_hash = crypto::bcrypt::generate('Test1234'), role = 'employee', organization = organization:athar, full_name = 'باسل خوري', user_type = 'volunteer', created_at = time::now();
CREATE account:at_vol2 SET email = 'nour@athar-team.org', password_hash = crypto::bcrypt::generate('Test1234'), role = 'employee', organization = organization:athar, full_name = 'نور الشامي', user_type = 'volunteer', created_at = time::now();

-- ══════════════════════════════════════════════════
-- TechFlow Solutions — شركة تقنية ناشئة
-- ══════════════════════════════════════════════════

-- ── TechFlow Departments ──
CREATE department:tf_eng SET name = 'قسم الهندسة', code = 'ENG', manager = 'خالد المصري', organization = organization:techflow, created_at = time::now();
CREATE department:tf_sales SET name = 'قسم المبيعات', code = 'SALES', manager = 'عمر النابلسي', organization = organization:techflow, created_at = time::now();
CREATE department:tf_product SET name = 'قسم المنتج', code = 'PROD', manager = 'ريم الخطيب', organization = organization:techflow, created_at = time::now();

-- ── TechFlow Employees ──
CREATE employee:tf_e1 SET name = 'خالد المصري', email = 'khalid@techflow.sy', phone = '+963955001001', role = 'developer', department = department:tf_eng, hire_date = '2024-01-15', status = 'active', base_salary = 2500000.0, housing_allowance = 500000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:techflow, created_at = time::now();
CREATE employee:tf_e2 SET name = 'لينا حداد', email = 'lina@techflow.sy', phone = '+963955001002', role = 'designer', department = department:tf_product, hire_date = '2024-03-01', status = 'active', base_salary = 2000000.0, housing_allowance = 400000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:techflow, created_at = time::now();
CREATE employee:tf_e3 SET name = 'عمر النابلسي', email = 'omar@techflow.sy', phone = '+963955001003', role = 'sales', department = department:tf_sales, hire_date = '2024-06-10', status = 'active', base_salary = 1800000.0, housing_allowance = 300000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:techflow, created_at = time::now();
CREATE employee:tf_e4 SET name = 'سامر قاسم', email = 'samer@techflow.sy', phone = '+963955001004', role = 'developer', department = department:tf_eng, hire_date = '2025-01-01', status = 'active', base_salary = 2200000.0, housing_allowance = 400000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:techflow, created_at = time::now();
CREATE employee:tf_e5 SET name = 'دانا يوسف', email = 'dana@techflow.sy', phone = '+963955001005', role = 'support', department = department:tf_product, hire_date = '2025-03-15', status = 'active', base_salary = 1500000.0, housing_allowance = 300000.0, transport_allowance = 150000.0, employment_type = 'full_time', organization = organization:techflow, created_at = time::now();

-- ── TechFlow Clients ──
CREATE client:tf_c1 SET company_name = 'شركة الأفق للتجارة الإلكترونية', name = 'فادي الحلبي', email = 'fadi@alofouq.com', phone = '+963944001001', address = 'دمشق - المزة', organization = organization:techflow, created_at = time::now();
CREATE client:tf_c2 SET company_name = 'مجموعة الريادة التقنية', name = 'هبة سعيد', email = 'heba@reyada.tech', phone = '+963944001002', address = 'حلب - الحمدانية', organization = organization:techflow, created_at = time::now();
CREATE client:tf_c3 SET company_name = 'مطاعم الشام الحديثة', name = 'طارق زيدان', email = 'tareq@alsham-rest.com', phone = '+963944001003', address = 'اللاذقية - الكورنيش', organization = organization:techflow, created_at = time::now();

-- ── TechFlow Invoices ──
CREATE invoice:tf_inv1 SET invoice_number = 'TF-2026-001', client = client:tf_c1, client_name = 'شركة الأفق للتجارة الإلكترونية', amount = 15000000.0, tax_amount = 750000.0, total = 15750000.0, status = 'paid', issue_date = '2026-01-15', due_date = '2026-02-15', organization = organization:techflow, created_at = time::now();
CREATE invoice:tf_inv2 SET invoice_number = 'TF-2026-002', client = client:tf_c2, client_name = 'مجموعة الريادة التقنية', amount = 25000000.0, tax_amount = 1250000.0, total = 26250000.0, status = 'pending', issue_date = '2026-02-01', due_date = '2026-03-01', organization = organization:techflow, created_at = time::now();
CREATE invoice:tf_inv3 SET invoice_number = 'TF-2026-003', client = client:tf_c3, client_name = 'مطاعم الشام الحديثة', amount = 8000000.0, tax_amount = 400000.0, total = 8400000.0, status = 'overdue', issue_date = '2026-01-01', due_date = '2026-01-31', organization = organization:techflow, created_at = time::now();

-- ── TechFlow Inventory ──
CREATE warehouse:tf_w1 SET name = 'مستودع التقنية', code = 'TF-WH1', address = 'دمشق - منطقة المعارض', is_active = true, organization = organization:techflow, created_at = time::now();

CREATE inventory_item:tf_i1 SET name = 'شاشة حاسوب 27 بوصة', sku = 'MON-27-4K', category = 'spare_parts', unit = 'قطعة', quantity = 15, min_quantity = 5, unit_cost = 1200000.0, warehouse = warehouse:tf_w1, organization = organization:techflow, created_at = time::now();
CREATE inventory_item:tf_i2 SET name = 'لوحة مفاتيح ميكانيكية', sku = 'KB-MECH-01', category = 'spare_parts', unit = 'قطعة', quantity = 30, min_quantity = 10, unit_cost = 350000.0, warehouse = warehouse:tf_w1, organization = organization:techflow, created_at = time::now();
CREATE inventory_item:tf_i3 SET name = 'كيبل شبكة Cat6 (100م)', sku = 'CBL-CAT6-100', category = 'consumables', unit = 'بكرة', quantity = 8, min_quantity = 3, unit_cost = 180000.0, warehouse = warehouse:tf_w1, organization = organization:techflow, created_at = time::now();
CREATE inventory_item:tf_i4 SET name = 'راوتر واي فاي 6', sku = 'RTR-WIFI6', category = 'spare_parts', unit = 'قطعة', quantity = 2, min_quantity = 5, unit_cost = 950000.0, warehouse = warehouse:tf_w1, organization = organization:techflow, created_at = time::now();

-- ── TechFlow Leave Requests ──
CREATE leave_request:tf_lr1 SET employee = account:tf_dev1, leave_type = 'annual', start_date = '2026-04-01', end_date = '2026-04-05', days = 5, status = 'approved', reason = 'إجازة عائلية', organization = organization:techflow, created_at = time::now();
CREATE leave_request:tf_lr2 SET employee = account:tf_dev2, leave_type = 'sick', start_date = '2026-03-20', end_date = '2026-03-21', days = 2, status = 'pending', reason = 'مراجعة طبية', organization = organization:techflow, created_at = time::now();

-- ── TechFlow CRM Opportunities ──
CREATE opportunity SET title = 'مشروع تطبيق توصيل', value = 50000000.0, stage = 'proposal', probability = 70, contact_name = 'فادي الحلبي', contact_email = 'fadi@alofouq.com', organization = organization:techflow, created_at = time::now();
CREATE opportunity SET title = 'نظام إدارة مطاعم', value = 30000000.0, stage = 'negotiation', probability = 85, contact_name = 'طارق زيدان', contact_email = 'tareq@alsham-rest.com', organization = organization:techflow, created_at = time::now();

-- ══════════════════════════════════════════════════
-- فريق أثر التطوعي
-- ══════════════════════════════════════════════════

-- ── Athar Departments ──
CREATE department:at_edu SET name = 'قسم التعليم', code = 'EDU', manager = 'لين العلي', organization = organization:athar, created_at = time::now();
CREATE department:at_media SET name = 'قسم الإعلام', code = 'MEDIA', manager = 'باسل خوري', organization = organization:athar, created_at = time::now();
CREATE department:at_field SET name = 'قسم العمل الميداني', code = 'FIELD', manager = 'نور الشامي', organization = organization:athar, created_at = time::now();

-- ── Athar Volunteers (as employees) ──
CREATE employee:at_e1 SET name = 'باسل خوري', email = 'bassel@athar-team.org', phone = '+963966001001', role = 'coordinator', department = department:at_media, hire_date = '2025-01-01', status = 'active', base_salary = 0.0, employment_type = 'volunteer', organization = organization:athar, created_at = time::now();
CREATE employee:at_e2 SET name = 'نور الشامي', email = 'nour@athar-team.org', phone = '+963966001002', role = 'volunteer', department = department:at_field, hire_date = '2025-02-15', status = 'active', base_salary = 0.0, employment_type = 'volunteer', organization = organization:athar, created_at = time::now();
CREATE employee:at_e3 SET name = 'ريان حمود', email = 'rayan@athar-team.org', phone = '+963966001003', role = 'volunteer', department = department:at_edu, hire_date = '2025-03-01', status = 'active', base_salary = 0.0, employment_type = 'volunteer', organization = organization:athar, created_at = time::now();
CREATE employee:at_e4 SET name = 'سلمى الأحمد', email = 'salma@athar-team.org', phone = '+963966001004', role = 'volunteer', department = department:at_field, hire_date = '2025-06-01', status = 'active', base_salary = 0.0, employment_type = 'volunteer', organization = organization:athar, created_at = time::now();

-- ── Athar Training Programs ──
CREATE training_program SET title = 'برنامج الإسعافات الأولية', description = 'تدريب المتطوعين على الإسعافات الأولية الأساسية', duration_hours = 20, status = 'active', trainer = 'د. سامر عبدالله', organization = organization:athar, created_at = time::now();
CREATE training_program SET title = 'مهارات التواصل الفعال', description = 'تطوير مهارات التواصل مع المجتمع المحلي', duration_hours = 12, status = 'active', trainer = 'أ. ريم خليل', organization = organization:athar, created_at = time::now();

-- ── Athar Leave Requests ──
CREATE leave_request:at_lr1 SET employee = account:at_vol1, leave_type = 'annual', start_date = '2026-04-10', end_date = '2026-04-12', days = 3, status = 'pending', reason = 'مناسبة عائلية', organization = organization:athar, created_at = time::now();

-- ══════════════════════════════════════════════════
-- Actuators — بيانات المنصة
-- ══════════════════════════════════════════════════

-- ── Actuators Departments ──
CREATE department:act_eng SET name = 'قسم التطوير', code = 'DEV', manager = 'يحيى سوان', organization = organization:actuators, created_at = time::now();
CREATE department:act_ops SET name = 'قسم العمليات', code = 'OPS', manager = 'محمود الأحمد', organization = organization:actuators, created_at = time::now();

-- ── Actuators Employees ──
CREATE employee:act_e1 SET name = 'محمود الأحمد', email = 'mahmoud@actuators.app', phone = '+963933000001', role = 'manager', department = department:act_ops, hire_date = '2023-09-01', status = 'active', base_salary = 3000000.0, housing_allowance = 600000.0, transport_allowance = 250000.0, employment_type = 'full_time', organization = organization:actuators, created_at = time::now();
CREATE employee:act_e2 SET name = 'سارة محمد', email = 'sara@actuators.app', phone = '+963933000002', role = 'developer', department = department:act_eng, hire_date = '2024-03-01', status = 'active', base_salary = 2500000.0, housing_allowance = 500000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:actuators, created_at = time::now();
CREATE employee:act_e3 SET name = 'عمر حسن', email = 'omar@actuators.app', phone = '+963933000003', role = 'developer', department = department:act_eng, hire_date = '2024-06-10', status = 'active', base_salary = 2200000.0, housing_allowance = 400000.0, transport_allowance = 200000.0, employment_type = 'full_time', organization = organization:actuators, created_at = time::now();

-- ── Actuators Clients ──
CREATE client:act_c1 SET company_name = 'شركة النور للتجارة', name = 'أحمد النور', email = 'ahmed@alnour.com', phone = '+963944002001', address = 'دمشق - باب توما', organization = organization:actuators, created_at = time::now();
CREATE client:act_c2 SET company_name = 'مصنع الشرق للبلاستيك', name = 'حسام الشرقي', email = 'hussam@sharq-plastic.com', phone = '+963944002002', address = 'حلب - الشيخ نجار', organization = organization:actuators, created_at = time::now();

-- ── Actuators Inventory ──
CREATE warehouse:act_w1 SET name = 'المستودع الرئيسي', code = 'ACT-WH1', address = 'دمشق - المنطقة الصناعية', is_active = true, organization = organization:actuators, created_at = time::now();

CREATE inventory_item:act_i1 SET name = 'محرك سيرفو 180°', sku = 'SRV-180', category = 'spare_parts', unit = 'قطعة', quantity = 50, min_quantity = 15, unit_cost = 85000.0, warehouse = warehouse:act_w1, organization = organization:actuators, created_at = time::now();
CREATE inventory_item:act_i2 SET name = 'حساس درجة حرارة PT100', sku = 'SNS-PT100', category = 'spare_parts', unit = 'قطعة', quantity = 20, min_quantity = 10, unit_cost = 45000.0, warehouse = warehouse:act_w1, organization = organization:actuators, created_at = time::now();
CREATE inventory_item:act_i3 SET name = 'لوحة تحكم Arduino Mega', sku = 'ARD-MEGA', category = 'spare_parts', unit = 'قطعة', quantity = 3, min_quantity = 5, unit_cost = 120000.0, warehouse = warehouse:act_w1, organization = organization:actuators, created_at = time::now();

-- ── Actuators Invoices ──
CREATE invoice:act_inv1 SET invoice_number = 'ACT-2026-001', client = client:act_c1, client_name = 'شركة النور للتجارة', amount = 5000000.0, tax_amount = 250000.0, total = 5250000.0, status = 'paid', issue_date = '2026-02-01', due_date = '2026-03-01', organization = organization:actuators, created_at = time::now();
CREATE invoice:act_inv2 SET invoice_number = 'ACT-2026-002', client = client:act_c2, client_name = 'مصنع الشرق للبلاستيك', amount = 12000000.0, tax_amount = 600000.0, total = 12600000.0, status = 'pending', issue_date = '2026-03-01', due_date = '2026-04-01', organization = organization:actuators, created_at = time::now();
