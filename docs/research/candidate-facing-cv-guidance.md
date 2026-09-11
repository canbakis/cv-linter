# Candidate-facing CV guidance notes

**Date:** 11 September 2026
**Status:** Hypothesis and fixture-design input; not authority for universal ATS behavior, lint severity, or numeric thresholds.

## Reviewed sources

- Microsoft Word's [How to write a resume with AI in Word](https://word.cloud.microsoft/create/en/blog/write-resume-ai/) (updated 25 May 2026) tells authors to replace template example text, verify generated job titles, dates, and achievements, and review AI-written content for factual accuracy before submission.
- Microsoft Word's [The best resume format in 2026](https://word.cloud.microsoft/create/en/blog/best-resume-formats/) (updated 27 July 2026) recommends clear structure, concise bullets in place of paragraphs for duties and achievements, and converting long paragraphs into scannable bullet points.
- Harvard FAS Mignone Center for Career Success, [AI for Resumes and Cover Letters](https://careerservices.fas.harvard.edu/ai-resumes-and-cover-letters/), presents generative AI as an editing aid rather than the primary author. It recommends accurate and authentic user-reviewed suggestions, job-specific context, basic formatting, and caution with personal or proprietary data.

## Bounded implications for CV Linter

These are candidate-facing career-guidance sources, not ATS parser specifications or empirical studies. They support human-review guidance around factual accuracy, authenticity, privacy, structure, concision, and scannability. They do not establish:

- how often unfinished template content appears in submitted CVs;
- a deterministic vocabulary for detecting placeholder text;
- the current 1,200-character `cv.structure.dense_block` threshold;
- a universal keyword strategy or formatting requirement;
- compatibility with, ranking by, or outcomes from any ATS.

Use the articles to motivate test questions, representative fixtures, and cautious host-AI advice. Any deterministic default or numeric threshold still needs an explicit project rationale and local validation. The removed `cv.literal.placeholder_text` heuristic should not be reintroduced solely on the strength of these sources.
