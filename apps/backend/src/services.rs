use pqc_common::types::Assessment;

pub fn summarize_assessment(assessments: &[Assessment]) -> i32 {
    if assessments.is_empty() {
        return 0;
    }

    assessments.iter().map(|a| a.score).sum::<i32>() / assessments.len() as i32
}
