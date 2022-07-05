def get_page_category(page_slug):
    parts = page_slug.split(":")
    if len(parts) == 1:
        return "_default"

    return parts[0]
