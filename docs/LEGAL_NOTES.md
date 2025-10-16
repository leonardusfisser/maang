# Legal & Compliance Notes

## Entities
- Data Processor: Estonian OÜ (owns the app)
- Parent: Cyprus Holding Ltd
- Owner: Individual (Cyprus tax resident)

## Jurisdictions
- Data stored in EU (Hetzner DE/FI)
- Tenants: English-speaking markets (EU, UK, US, CA, ZA, AU, NZ)
- Estonian OÜ acts as data processor under GDPR.

## Sub-processors
| Service | Purpose | Region | Notes |
|----------|----------|--------|-------|
| Hetzner Online GmbH | Hosting | EU | GDPR compliant |
| Cloudflare | CDN | Global | Data cached worldwide |
| Stripe Payments Europe Ltd | Billing | EU | PCI-DSS compliant |
| Mailgun | Email | EU / US | Optional, SCC in place |

## Data Processing Agreement
Each tenant signs DPA:
- You (OÜ) = Processor
- Tenant = Controller
- Sub-processors listed here.

## Privacy Policy / ToS
Maintain current versions at `/legal/privacy.html` and `/legal/terms.html` on your website.
