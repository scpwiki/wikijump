<?php

declare(strict_types=1);

namespace Wikijump\Http\Controllers;

use Illuminate\Contracts\View\View;
use Illuminate\Support\Facades\URL;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Helpers\LegacyTools;

class PageController extends Controller
{
    /** Returns a `view` for the current page. */
    public function show(): View
    {
      $values = LegacyTools::generateScreenVars();

      $title = null;
      $canonical = URL::current();
      $license = null;
      $social_title = null;
      $sidebar_content = null;
      $page_content = null;
      $slug = null;
      $category = null;
      $breadcrumbs = null;
      $page_title = null;
      $revision = null;
      $last_edit_date = null;
      $last_edit_days_since = null;
      $tags = null;

      if ($values['site']) {
        $title = $values['site']->getName();
      }

      if ($values['wikiPage']) {
          $page = $values['wikiPage'];
          $timestamp = $page->getDateLastEdited()->getTimestamp();

          $sidebar_content = $values['sideBar1Content'] ?? null;
          $page_content = $values['pageContent'] ?? null;

          $slug = $page->getUnixName();
          $page_title = $page->getTitleOrUnixName();
          $revision = $page->getRevisionNumber();
          $last_edit_date = strftime('%x %r', $timestamp);
          $last_edit_days_since = floor((time() - $timestamp) / (60 * 60 * 24));
          $tags = $values['tags'] ?? null;

          $title = $page_title . ' | ' . $title;
          $social_title = $page_title;

          // this should always be there, but just in case...
          if ($values['category']) {
              $category = $values['category']->getName();

              // we only want to provide license info if the page actually has one
              $lic = $values['category']->getLicense();
              if ($lic->url()) {
                  $license = $lic;
              }
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
          'canonical' => $canonical,
          'license' => $license,

          'social_title' => $social_title,
          'social_type' => 'article',
          'social_url' => $canonical,

          'header_img_url' => '/files--static/media/logo-outline.min.svg',
          'sidebar_content' => $sidebar_content,

          'page_content' => $page_content,
          'page_slug' => $slug,
          'page_category' => $category,
          'page_title' => $page_title,
          'page_breadcrumbs' => $breadcrumbs,
          'page_revision' => $revision,
          'page_last_edit_date' => $last_edit_date,
          'page_last_edit_days_since' => $last_edit_days_since,
          'page_tags' => $tags,

          'HTTP_SCHEMA' => GlobalProperties::$HTTP_SCHEMA,
          'URL_DOMAIN' => GlobalProperties::$URL_DOMAIN,
          'URL_HOST' => GlobalProperties::$URL_HOST,
          'SERVICE_NAME' => GlobalProperties::$SERVICE_NAME,
      ]);
    }
}
