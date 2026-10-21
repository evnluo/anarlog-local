# Capability matrix

| Capability | Anarlog Local v1 | Notes |
| --- | --- | --- |
| Meeting/microphone capture | Included | Local native capture; macOS permissions required |
| Transcript and notes editing | Included | Stored locally |
| Audio playback and 0.5x–2x speed | Included | No account or plan check |
| Local export/import | Included | Markdown and supported local formats |
| On-device STT | Included | Soniqo, Apple Speech, or user-selected local model where supported |
| BYOK STT | Included | Direct provider endpoint/key selected by the user |
| On-device/local LLM | Included | Apple Foundation Models, Ollama, LM Studio, and compatible endpoints |
| BYOK LLM chat and summaries | Included | Direct provider endpoint/key selected by the user |
| Personal dictionary | Included | No account or plan check |
| System dictation | Included | Local lifecycle identity; preserves permission, cancellation, and insertion checks |
| Custom summary format/templates | Included | No account or plan check |
| Local automation engine | Included | Local workflows such as Markdown export |
| Anarlog managed AI | Excluded | Provider hidden and vendor domains blocked |
| Anarlog managed web search | Excluded | Hosted search tool is not registered with local chat |
| Billing, trials, referrals | Excluded | No subscription UI |
| Anarlog cloud sync | Excluded | Cloud lifecycle not mounted |
| Cloud sharing/shared notes | Excluded | Sharing lifecycle and CTA removed |
| Teams/workspaces/CRM | Excluded | Hosted SaaS surfaces removed from navigation |
| Product analytics and vendor crash reporting | Excluded | No keys embedded; analytics wrapper is a no-op |
| Vendor updater feed | Excluded | Updater disabled; release artifacts are installed manually |
| Hosted Google/Outlook integrations | Excluded from onboarding | Local Apple Calendar remains available where supported |

The repository retains some upstream modules to satisfy shared build dependencies. A retained module is not an enabled v1 capability: hosted lifecycles are not mounted, hosted navigation is removed, managed providers are hidden, and vendor HTTP domains are blocked in the webview.
