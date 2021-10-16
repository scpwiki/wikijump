#include <stdbool.h>
#include <stdio.h>
#include <inttypes.h>

#include <ftml.h>

static void print_page_refs(const char *name, const ftml_page_ref **list, size_t len)
{
	printf("%s:\n", name);

	for (size_t i = 0; i < len; i++) {
		const ftml_page_ref *page_ref = list[i];
		const char *site = page_ref->site;
		if (!site) {
			site = "(default)";
		}

		printf("  - %s %s\n", site, page_ref->page);
	}
}

static void print_strings(const char *name, const char **list, size_t len)
{
	printf("%s:\n", name);

	if (len == 0) {
		printf("    (none)\n");
	}

	for (size_t i = 0; i < len; i++) {
		printf("  - %s\n", list[i]);
	}
}

static const char *meta_type(ftml_html_meta_type type)
{
	switch (type) {
	case META_NAME:
		return "Name";
	case META_HTTP_EQUIV:
		return "Http-Equiv";
	case META_PROPERTY:
		return "Property";
	default:
		/* Error */
		return NULL;
	}
}

int main(int argc, char **argv)
{
	struct ftml_html_output output;
	struct ftml_page_info page_info = {
		.page = "my-page",
		.category = NULL,
		.site = "www",
		.title = "Test page!",
		.alt_title = NULL,
		.rating = 69.0,
		.tags_list = NULL,
		.tags_len = 0,
		.language = "default",
	};
	const ftml_wikitext_settings settings = {
		.mode = WIKITEXT_MODE_PAGE,
		.enable_page_syntax = true,
		.use_true_ids = true,
		.allow_local_paths = true,
	};
	const char *input = (
		"[[css]]\n"
		"div.blockquote { color: blue; }\n"
		"[[/css]]\n"
		"**Test**\n"
		"[[module CSS]]\n"
		".my-class {\n"
		"    display: block;\n"
		"}\n"
		"[[/module]]\n"
		"__string__\n"
	);

	ftml_render_html(&output, input, &page_info, &settings);

	printf("Input:\n%s\n----\n\n", input);
	printf("Body:\n%s\n----\n\n", output.body);
	printf("Styles:\n");
	for (size_t i = 0; i < output.styles_len; i++) {
		printf("%s\n", output.styles_list[i]);

		if (i < output.styles_len - 1) {
			printf("----\n");
		} else {
			printf("\n\n");
		}
	}

	printf("Meta Fields:\n");
	for (size_t i = 0; i < output.meta_len; i++) {
		struct ftml_html_meta *meta = &output.meta_list[i];

		printf("    Type: %s\n", meta_type(meta->tag_type));
		printf("    Name: %s\n", meta->name);
		printf("    Value: %s\n", meta->value);

		if (i < output.meta_len - 1) {
			printf("    ----\n");
		} else {
			printf("\n\n");
		}
	}

	printf("Warnings:\n");
	for (size_t i = 0; i < output.warning_len; i++) {
		struct ftml_warning *warn = &output.warning_list[i];

		printf("    Token: %s\n", warn->token);
		printf("    Rule: %s\n", warn->rule);
		printf("    Span: %zu..%zu\n", warn->span_start, warn->span_end);
		printf("    Kind: %s\n", warn->kind);

		if (i < output.warning_len - 1) {
			printf("    ----\n");
		}
	}

	printf("Backlinks:\n----\n\n");
	print_page_refs(
		"Included pages",
		output.backlinks.included_pages_list,
		output.backlinks.included_pages_len
	);
	print_page_refs(
		"Internal links",
		output.backlinks.internal_links_list,
		output.backlinks.internal_links_len
	);
	print_strings(
		"External links",
		output.backlinks.external_links_list,
		output.backlinks.external_links_len
	);

	return 0;
}
