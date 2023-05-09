use scryfall::set::Set;
use ::scryfall::bulk::default_cards;

pub(crate) async fn fetch_sets() -> Option<Vec<Set>> {
    let set_list = match Set::all().await {
        Ok(set_list) => set_list,
        Err(e) => {
            eprintln!("Error fetching sets: {}", e);
            return None;
        }
    };

    let sets = set_list.into_inner().collect::<Vec<_>>();

    Some(sets)
}

pub(crate) async fn fetch_cards(
) -> Option<impl Iterator<Item = Result<scryfall::Card, scryfall::Error>>> {
    

    match default_cards().await {
        Ok(cards) => Some(cards),
        Err(e) => {
            eprintln!("Error fetching cards: {}", e);
            None
        }
    }
}
