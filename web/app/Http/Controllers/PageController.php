<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\View\View;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Services\License\License;
use Wikijump\Services\License\LicenseMapping;

class PageController extends Controller
{
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
