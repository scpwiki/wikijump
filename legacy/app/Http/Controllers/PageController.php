<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Contracts\View\View;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Wikijump\Common\APIError;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Services\Deepwell\Models\Category;
use Wikijump\Services\Deepwell\Models\Page;
use Wikijump\Services\Deepwell\Models\User;
use Wikijump\Services\License\License;
use Wikijump\Services\License\LicenseMapping;

class PageController extends Controller
{
    /** Guard used to handle authentication. */
    private StatefulGuard $guard;

    /**
     * @param StatefulGuard $guard
     */
    public function __construct(StatefulGuard $guard)
    {
        $this->guard = $guard;
    }

    private function resolvePageType(Request $request): ?array
    {
        $output_type = (string) $request->query('type', 'none');

        $metadata = false;
        $wikitext = false;
        $html = false;

        switch ($output_type) {
            case 'all':
                $metadata = true;
                $wikitext = true;
                $html = true;
                break;
            case 'metadata':
                $metadata = true;
                break;
            case 'metadata-html':
                $metadata = true;
                $html = true;
                break;
            case 'metadata-wikitext':
                $metadata = true;
                $wikitext = true;
                break;
            case 'wikitext':
                $wikitext = true;
                break;
            case 'html':
                $html = true;
                break;
            case 'none':
                break;
            default:
                return null;
        }

        return [
            'metadata' => $metadata,
            'wikitext' => $wikitext,
            'html' => $html,
        ];
    }

    /**
     * @return Page|Response
     */
    private function resolvePage(
        string $path_type,
        string $path,
        bool $wikitext = false,
        bool $html = false
    ) {
        $site = LegacyTools::getCurrentSite();
        if ($site === null) {
            return apierror(404, APIError::SITE_NOT_FOUND);
        }

        $site_id = $site->getSiteId();

        $page = null;

        if ($path_type === 'slug') {
            $page = Page::findSlug($site_id, $path, $wikitext, $html);
        } elseif ($path_type === 'id') {
            $page = Page::findId($site_id, (int) $path, $wikitext, $html);
        } else {
            return apierror(400, APIError::INVALID_PAGE_PATH);
        }

        if ($page === null) {
            return apierror(404, APIError::PAGE_NOT_FOUND);
        }

        return $page;
    }

    /**
     * Gets a page.
     * Endpoint: `GET:/page/{path_type}/{path}` | `pageGet`
     */
    public function pageGet(Request $request, string $path_type, string $path)
    {
        $output_type = $this->resolvePageType($request);
        if ($output_type === null) {
            return apierror(400, APIError::INVALID_PAGE_TYPE);
        }

        $page = $this->resolvePage(
            $path_type,
            $path,
            $output_type['wikitext'],
            $output_type['html'],
        );

        // api error was returned
        if ($page instanceof Response) {
            return $page;
        }

        $output = [];

        if ($output_type['metadata']) {
            $avatars = (bool) $request->query('avatars', false);

            // TODO: get creator through earliest revision
            // (unless we implement the authors field first)

            $updater = User::findId($page->revision_user_id);

            if ($updater !== null) {
                $updater = $updater->toApiArray($avatars);
            }

            $output = array_merge($output, [
                'id' => $page->id,
                'slug' => $page->slug,
                'category' => $page->page_category_id,
                'parent' => null, // TODO
                'children' => [], // TODO
                'title' => $page->title,
                'altTitle' => $page->alt_title,
                'tags' => $page->tags,
                'score' => 0, // TODO
                'created' => $page->page_created_at,
                'creator' => null, // TODO
                'updated' => $page->revision_created_at,
                'updater' => $updater,
            ]);
        }

        if ($output_type['wikitext']) {
            $output['wikitext'] = $page->wikitext;
        }

        if ($output_type['html']) {
            $output['html'] = $page->compiled_html;
        }

        return new Response($output, 200);
    }

    /** Returns a `view` for the current page. */
    public function show(?string $path = null): View
    {
        // TODO: description, image, twitter, etc.
        // TODO: site theming
        // TODO: favicons
        // TODO: header image/text + subtitle management
        // TODO: navbar items
        // TODO: private sites
        // TODO: page queries, like ?noredirect=true
        // TODO: breadcrumbs

        $site = LegacyTools::getCurrentSite();
        if ($site === null) {
            abort(404);
        }

        $slug = LegacyTools::redirectToNormalUrl($site, $path ?? '', '');

        $site_id = $site->getSiteId();

        $page = Page::findSlug($site_id, $slug, false, true);
        if ($page === null) {
            abort(404);
        }

        $title = null;
        $alt_title = null;
        $license = null;
        $social_title = null;
        $sidebar_content = null;
        $page_content = null;
        $category = null;
        $breadcrumbs = null;
        $title = null;
        $revision = null;
        $timestamp = null;
        $tags = null;

        $title = $site->getName();

        // TODO: this doesn't seem elegant?
        $sidebar_page = Page::findSlug($site_id, 'nav:side', false, true);
        if ($sidebar_page !== null) {
            $sidebar_content = $sidebar_page->compiled_html;
        }

        $page_content = $page->compiled_html;
        $tags = $page->tags;
        $revision = $page->revision_number;
        $timestamp = $page->lastUpdated()->getTimestamp();
        $title = $page->title;
        $alt_title = $page->alt_title;
        $social_title = $title;

        $category_data = Category::findIdOnly($page->page_category_id);
        if ($category_data !== null) {
            $category = $category_data->slug;
            $license = LicenseMapping::get('cc_by_sa_3'); // TODO hardcoded
        }

        return view('next.wiki.page', [
            'title' => $title,
            'license' => $license,

            'social_title' => $social_title,
            'social_type' => 'article',

            'sidebar_content' => $sidebar_content,

            'page_content' => $page_content,
            'page_category' => $category,
            'page_title' => $title,
            'page_alt_title' => $alt_title,
            'page_breadcrumbs' => $breadcrumbs,
            'page_revision' => $revision,
            'page_last_edit_timestamp' => $timestamp,
            'page_tags' => $tags,
        ]);
    }
}
