use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub const CATEGORIES: [&str; 14] = [
    "Elektronica",
    "Gaming",
    "Boodschappen",
    "Mode & Accessoires",
    "Beauty & Gezondheid",
    "Familie & Kinderen",
    "Home & Living",
    "Tuin & Doe-het-zelf",
    "Auto & Motor",
    "Cultuur & Vrije tijd",
    "Sport & Outdoor",
    "Telecom & Internet",
    "Geldzaken & Verzekeringen",
    "Services & Contracten",
];

pub fn match_category(category: &str) -> Option<String> {
    if category.eq("") {
        return None;
    }

    let matcher = SkimMatcherV2::default();
    let mut category_match: Option<String> = None;

    for pepper_category in CATEGORIES {
        if matcher.fuzzy_match(pepper_category, category).is_some() {
            category_match = Some(pepper_category.to_lowercase());
            break;
        }
    }

    category_match
}
