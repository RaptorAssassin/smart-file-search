pub struct Prompts {
    pub keywords: &'static str,
    pub summary: &'static str,
    pub image_caption: &'static str,
}

pub const PROMPTS: Prompts = Prompts {
    keywords: "Extract 5-10 short keywords from the following text. Reply ONLY with a valid JSON array of strings, for example: [\"keyword1\", \"keyword2\"]. No explanations, introductions, no extra text.",
    summary: "Provide a one-line summary of the following text. Reply ONLY with the summary and no additional text.",
    image_caption: "Generate a concise caption for the following image. Reply ONLY with the caption and no additional text. No explanations, introductions, or extra text.",
};
