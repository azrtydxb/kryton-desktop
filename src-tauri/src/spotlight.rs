#![cfg(target_os = "macos")]

use objc2::{rc::Retained, AnyThread};
use objc2_core_spotlight::{
    CSSearchableIndex, CSSearchableItem, CSSearchableItemAttributeSet,
};
use objc2_foundation::{NSArray, NSString};

/// Index or update a note in macOS Spotlight.
pub fn index_note(
    account_id: &str,
    note_path: &str,
    title: &str,
    snippet: &str,
) -> Result<(), String> {
    unsafe {
        let content_type = NSString::from_str("public.text");
        // initWithItemContentType is deprecated but remains available; the
        // `objc2-uniform-type-identifiers` feature is not enabled so we use it.
        #[allow(deprecated)]
        let attrs = CSSearchableItemAttributeSet::initWithItemContentType(
            CSSearchableItemAttributeSet::alloc(),
            &content_type,
        );
        attrs.setTitle(Some(&NSString::from_str(title)));
        attrs.setContentDescription(Some(&NSString::from_str(snippet)));

        let unique_id =
            NSString::from_str(&format!("kryton-{}-{}", account_id, note_path));
        let domain_id = NSString::from_str(&format!("account-{}", account_id));

        let item = CSSearchableItem::initWithUniqueIdentifier_domainIdentifier_attributeSet(
            CSSearchableItem::alloc(),
            Some(&unique_id),
            Some(&domain_id),
            &attrs,
        );

        let items: Retained<NSArray<CSSearchableItem>> =
            NSArray::from_retained_slice(&[item]);
        let index = CSSearchableIndex::defaultSearchableIndex();
        index.indexSearchableItems_completionHandler(&items, None);
    }
    Ok(())
}

/// Remove a single note from Spotlight by its unique identifier.
pub fn remove_note(account_id: &str, note_path: &str) -> Result<(), String> {
    unsafe {
        let unique_id =
            NSString::from_str(&format!("kryton-{}-{}", account_id, note_path));
        let ids: Retained<NSArray<NSString>> =
            NSArray::from_retained_slice(&[unique_id]);
        let index = CSSearchableIndex::defaultSearchableIndex();
        index.deleteSearchableItemsWithIdentifiers_completionHandler(&ids, None);
    }
    Ok(())
}

/// Remove all notes for an account from Spotlight by domain identifier.
pub fn clear_account(account_id: &str) -> Result<(), String> {
    unsafe {
        let domain_id = NSString::from_str(&format!("account-{}", account_id));
        let domains: Retained<NSArray<NSString>> =
            NSArray::from_retained_slice(&[domain_id]);
        let index = CSSearchableIndex::defaultSearchableIndex();
        index.deleteSearchableItemsWithDomainIdentifiers_completionHandler(
            &domains,
            None,
        );
    }
    Ok(())
}
