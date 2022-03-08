<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Contracts\View\View;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Wikijump\Common\APIError;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Services\Deepwell\Models\Page;
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

    /**
     * @return null|int|string
     */
    private function resolvePagePath(Request $request)
    {
        $path_type = (string) $request->input('path_type');
        $path = (string) $request->input('path');
        if ($path_type !== 'slug' || $path_type !== 'id') {
            return null;
        }
        return $path_type === 'slug' ? $path : (int) $path;
    }

    private function resolvePageType(Request $request): ?array
    {
        $page_type = (string) $request->input('type', 'none');

        $metadata = false;
        $wikitext = false;
        $html = false;

        switch ($page_type) {
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
     * @param int|string $slug_or_id
     * @param int|string $site_id
     */
    private function resolvePage(
        $slug_or_id,
        $site_id,
        bool $wikitext = false,
        bool $html = false
    ): ?Page {
        $page = null;
        if (typeof($slug_or_id) === 'string') {
            $page = Page::findSlug($site_id, $slug_or_id, $wikitext, $html);
        } else {
            $page = Page::findId($site_id, $slug_or_id, $wikitext, $html);
        }
        return $page;
    }

    public function pageGet(Request $request)
    {
        $site = LegacyTools::getCurrentSite();
        if ($site === null) {
            return apierror(404, APIError::SITE_NOT_FOUND);
        }

        $site_id = $site->getSiteId();

        $page_path = $this->resolvePagePath($request);
        if ($page_path === null) {
            return apierror(400, APIError::INVALID_PAGE_PATH);
        }

        $page_type = $this->resolvePageType($request);
        if ($page_type === null) {
            return apierror(400, APIError::INVALID_PAGE_TYPE);
        }

        [$metadata, $wikitext, $html] = $page_type;

        $page = $this->resolvePage($site_id, $page_path, $wikitext, $html);
        if ($page === null) {
            return apierror(404, APIError::PAGE_NOT_FOUND);
        }

        $output = [];

        if ($metadata) {
            array_merge($output, [
                'id' => $page->id(),
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
                'updated' => $page->page_updated_at,
                'updater' => null, // TODO
            ]);
        }

        if ($wikitext) {
            array_merge($output, [
                'wikitext' => $page->wikitext,
            ]);
        }

        if ($html) {
            array_merge($output, [
                'html' => $page->compiled_html,
            ]);
        }

        return new Response($output, 200);
    }

    /** Returns a `view` for the current page. */
    public function show(): View
    {
        $values = LegacyTools::generateScreenVars();

        $title = null;
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

        if ($values['site']) {
            $title = $values['site']->getName();
        }

        if ($values['wikiPage']) {
            $page = $values['wikiPage'];

            $sidebar_content = $values['sideBar1Content'] ?? null;
            $page_content = $values['pageContent'] ?? null;
            $tags = $values['tags'] ?? null;
            $revision = $page->revision_number;
            $timestamp = $page->lastUpdated()->getTimestamp();
            $title = $page->title;
            $license = LicenseMapping::get('cc_by_sa_3'); // TODO hardcoded
            $social_title = $title;

            // this should always be there, but just in case...
            if ($values['category']) {
                $category = $values['category']->getName();
            }

            if ($values['breadcrumbs']) {
                $breadcrumbs = [];
                foreach ($values['breadcrumbs'] as $breadcrumb) {
                    $breadcrumbs[] = [
                        'title' => $breadcrumb->getTitleOrUnixName(),
                        'slug' => $breadcrumb->getUnixName(),
                    ];
                }
            }
        }

        return view('next.wiki.page', [
            // TODO: description, image, twitter, etc.
            // TODO: site theming
            // TODO: favicons
            // TODO: header image/text + subtitle management
            // TODO: navbar items

            'title' => $title,
            'license' => $license,

            'social_title' => $social_title,
            'social_type' => 'article',

            'sidebar_content' => $sidebar_content,

            'page_content' => $page_content,
            'page_category' => $category,
            'page_title' => $title,
            'page_breadcrumbs' => $breadcrumbs,
            'page_revision' => $revision,
            'page_last_edit_timestamp' => $timestamp,
            'page_tags' => $tags,
        ]);
    }
}
