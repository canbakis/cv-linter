# CV writing and AI-review guidance

Read this reference for qualitative CV review, job-specific writing advice, or rewrite suggestions. Do not use it to create deterministic lint findings.

## Practical guidance

- Treat AI output as a proposed edit, not as a fact source or final product. Start from the supplied CV, keep the author's voice, and require the user to review the result for accuracy and authenticity.
- Tailor emphasis and wording to the supplied job description, but include keywords only where the CV provides evidence that they apply.
- Prefer clear structure and concise, scannable bullets when that improves the author's material. Do not turn general readability advice into a fixed length, layout, or character-count rule.
- Preserve names, employers, roles, dates, credentials, numbers, skills, responsibilities, and impact claims unless the user supplies a correction.
- Minimize personal or proprietary content sent to AI systems and retain the Skill's local-tool-versus-host-model privacy distinction.

## Review checklist

Use this checklist for a general CV check. Report only observations grounded in the selected CV, and treat each item as contextual guidance rather than a pass/fail rule:

- **Structure:** recognizable sections, consistent chronology, and an information order that makes the candidate's recent and relevant experience easy to find.
- **Scanability:** concise paragraphs or bullets, consistent formatting, and limited repetition. Do not impose a universal page, bullet, or character count.
- **Clarity:** concrete roles, responsibilities, technologies, and outcomes. Suggest stronger action or impact wording only when the supplied text supports it.
- **Evidence:** achievements and scope are specific enough to understand without inventing metrics or causality.
- **Consistency:** names, dates, tense, capitalization, and terminology do not conflict across sections.
- **Relevance:** emphasis matches the user's target role or supplied job description; never add unsupported keywords.
- **Contact and context:** expected contact details and essential employer, role, education, or credential context are readable in extraction. Absence in extracted text is an observation, not proof that the visual document omits it.

These principles are supported as candidate-facing guidance by:

- Microsoft Word, [How to write a resume with AI in Word](https://word.cloud.microsoft/create/en/blog/write-resume-ai/), which recommends replacing example content and reviewing generated details for accuracy.
- Microsoft Word, [The best resume format in 2026](https://word.cloud.microsoft/create/en/blog/best-resume-formats/), which recommends clear structure, concise bullets, and converting long paragraphs into more scannable content.
- Harvard FAS Mignone Center for Career Success, [AI for Resumes and Cover Letters](https://careerservices.fas.harvard.edu/ai-resumes-and-cover-letters/), which describes generative AI as an editing aid rather than the primary author and emphasizes authentic, accurate, deliberately reviewed suggestions and privacy caution.

## Evidence boundary

These sources are writing guidance, not ATS parser specifications or empirical validation of CV Linter rules. They do not justify an ATS score, hiring prediction, universal keyword list, vendor-compatibility claim, deterministic severity, or numeric threshold. Present formatting and wording recommendations as host-AI advice, separate from the Rust linter's findings.
