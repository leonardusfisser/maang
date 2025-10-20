Admin Dashboard Design - Lillpepe
Architecture
Route Prefix: /admin
Auth: Admin-only middleware (separate from white label auth)
Templates: Askama HTML templates in templates/admin/
Styling: Single CSS file with variables, minimal JavaScript

Pages Structure
1. Dashboard Overview (/admin)
   Metrics Cards (4-column grid):

White Labels (active) - Green badge - Count + Click → filter list
Gray Labels (payment overdue) - Yellow badge - Count + Click → filter list
Black Labels (defaulted) - Red badge - Count + Click → filter list
Red Labels (deleted/archived) - Gray badge - Count + Click → archive view

Revenue Section:

Total MRR (Monthly Recurring Revenue)
MRR growth % vs last month
Failed payments count (urgent - red highlight)

System Health:

Total storage used / available
Database connection status per white label
Binary version deployed
Last backup timestamp

Recent Activity Feed:

Last 10 white label actions (signup, payment, suspension, deletion)
Timestamp, domain, action, admin who performed it


2. White Labels List (/admin/labels)
   Filters Bar:

Status dropdown: All / White / Gray / Black / Red
Search: Domain name
Sort: Created (newest/oldest), Last payment, Storage used

Table Columns:

Status Dot (color-coded circle)
Domain (clickable → detail page)
Created (YYYY-MM-DD)
Last Payment (YYYY-MM-DD or "Overdue")
Storage (MB used)
Actions (View | Suspend | Email | Delete)

Pagination: 50 per page

3. White Label Detail (/admin/labels/{id})
   Header:

Domain (large)
Status badge (White/Gray/Black/Red)
Quick actions: View Site | Suspend | Delete

Subscription Info:

Plan type
Price per billing cycle
Next billing date
Payment method (last 4 digits)
Link to Stripe customer

Usage Stats:

Storage: Images (MB), Videos (MB), Database (MB)
Total pages/posts/products count
Last login timestamp

Payment History Table:

Date, Amount, Status (Paid/Failed/Refunded), Stripe invoice link
Last 12 months

Activity Log:

Timestamp, Action (login, updated colors, uploaded media, etc.)
Last 50 actions

Danger Zone:

Suspend subscription
Delete white label (with confirmation modal)


4. Payments Dashboard (/admin/payments)
   Upcoming Renewals (Next 7 days):

Table: Domain, Amount, Renewal date, Status
Highlight any that might fail (payment method expiring)

Failed Payments (Action Required):

Table: Domain, Amount, Failed date, Retry count, Contact button
Sorted by most urgent

Revenue Chart:

Simple line graph: MRR over last 12 months
Data points clickable → monthly detail

Stripe Webhook Log:

Last 50 webhook events
Timestamp, Event type, Status (processed/failed), Raw JSON toggle


5. System Health (/admin/system)
   Binary Deployment:

Current version
Deployed timestamp
Rollback to previous version (button)

Database Status:

Per white label: Domain, DB size, Connection status, Last query
Alert if any connection failures

Error Log:

Last 100 errors across all white labels
Timestamp, Domain (if applicable), Error type, Message
Filter by severity (Error/Warning)

Backups:

Last backup: Timestamp, Size
Next scheduled backup
Manual backup trigger (button)

Storage Overview:

Total disk usage
Per white label breakdown (top 10 by usage)
Alert threshold warnings


Design Specifications
Color Scheme
css:root {
/* Status colors */
--white-label: #10b981;  /* green */
--gray-label: #f59e0b;   /* yellow/orange */
--black-label: #ef4444;  /* red */
--red-label: #6b7280;    /* gray */

/* UI colors */
--bg-primary: #ffffff;
--bg-secondary: #f9fafb;
--border: #e5e7eb;
--text-primary: #111827;
--text-secondary: #6b7280;
--accent: #3b82f6;       /* blue for links/actions */
}
Layout

Sidebar: Fixed left, 240px width

Logo/Title at top
Nav links: Dashboard, Labels, Payments, System, Logout


Main Content: Margin-left 240px, padding 2rem
Cards: White background, 1px border, 8px border-radius, subtle shadow
Tables: Striped rows, hover highlight, sticky header

Typography

Headers: System font stack (no custom fonts)
Body: 16px base, 1.5 line-height
Monospace: For IDs, domains, error messages

Responsive

Mobile: Sidebar collapses to top bar with hamburger
Tables: Horizontal scroll on small screens
Cards: Stack vertically on mobile


Technical Implementation
Askama Templates Structure
templates/admin/
├── base.html           (sidebar, header, common layout)
├── dashboard.html      (overview page)
├── labels_list.html    (table of white labels)
├── label_detail.html   (individual white label view)
├── payments.html       (payments dashboard)
├── system.html         (system health)
└── components/
├── status_badge.html
├── metric_card.html
└── table_row.html
Routes (Actix-Web)
rust// crates/application/web/src/admin/routes.rs
pub fn configure(cfg: &mut web::ServiceConfig) {
cfg.service(
web::scope("/admin")
.wrap(AdminAuth)  // Middleware
.route("", web::get().to(dashboard))
.route("/labels", web::get().to(labels_list))
.route("/labels/{id}", web::get().to(label_detail))
.route("/payments", web::get().to(payments))
.route("/system", web::get().to(system_health))
);
}
Data Queries

Use SurrealDB views/aggregations for metrics
Cache dashboard stats (Redis, 5min TTL)
Stream activity logs (don't load all in memory)

Security

Admin session separate from white label sessions
CSRF tokens on all forms
Rate limit admin endpoints (prevents brute force)
Audit log all admin actions


Nice-to-Have (Phase 2)

Export white label data as ZIP
Bulk operations (suspend multiple, email multiple)
Revenue forecasting
White label health score (login frequency, storage growth)
Custom alerts (webhook failures, payment issues)
Dark mode toggle


Implementation Priority

Week 1: Dashboard overview + Labels list
Week 2: Label detail page + Basic payments view
Week 3: System health + Polish
Week 4: Activity logging + Testing

Estimated: 4 weeks to production-ready admin dashboard