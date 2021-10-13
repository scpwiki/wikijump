<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\View\View;
use Faker;
use Wikijump\Services\License\LicenseMapping;

class PageMockedComposer
{

    private Faker\Generator $faker;

    /** Create a new mocked page composer. */
    public function __construct()
    {
        $this->faker = Faker\Factory::create();
    }

    /** Bind data to the view. */
    public function compose(View $view)
    {
        $f = $this->faker;

        $timestamp = $f->dateTimeThisYear->getTimestamp();

        $content = '';
        for ($i = 0; $i < $f->numberBetween(10, 100); $i++) {
            if ($f->boolean) {
                $content .= $f->paragraph;
            } else {
                $content .= '<p>' . $f->paragraph . '</p>';
            }
        }

        $content = '<wj-body class="wj-body">' . $content . '</wj-body>';

        $sidebar_content = null;
        if ($f->boolean) {
            $sidebar_content = '';
            for ($i = 0; $i < $f->numberBetween(1, 10); $i++) {
                if ($f->boolean) {
                    $sidebar_content .= '<h1>' . $f->streetName . '</h1>';
                } else {
                    $sidebar_content .=
                        '<div><a href="' . $f->url . '">' . $f->streetName . '</a></div>';
                }
            }

            $sidebar_content = '<wj-body class="wj-body">' . $sidebar_content . '</wj-body>';
        }

        $navbar_items = [];
        for ($i = 0; $i < $f->numberBetween(1, 5); $i++) {
            $title = $f->streetName;
            if ($f->boolean) {
                $link = $f->streetName;
                $navbar_items[$link] = [];
                for ($j = 0; $j < $f->numberBetween(1, 5); $j++) {
                    $navbar_items[$link][$f->streetName] = $f->url;
                }
            } else {
                $navbar_items[$title] = $f->url;
            }
        }

        $slug = $f->slug;
        $page_title = $f->streetName;
        $revision = $f->randomNumber(3);
        $last_edit_date = strftime('%x %r', $timestamp);
        $last_edit_days_since = floor((time() - $timestamp) / (60 * 60 * 24));
        $tags = $f->unique()->words(10);
        $title = $page_title . ' | Wikijump';
        $social_title = $page_title;
        $category = $f->word;

        $license = null;
        if ($f->boolean) {
            $list = config('licenses.raw');
            $id = $list[array_rand($list)]['id'];
            $license = LicenseMapping::get($id);
        }

        $breadcrumbs = [];
        if ($f->boolean) {
            for ($i = 0; $i < $f->numberBetween(1, 3); $i++) {
                $breadcrumbs[] = [
                    'title' => $f->streetName,
                    'url' => $f->url,
                ];
            }
        }

        $view
            ->with('title', $title)
            ->with('social_title', $social_title)
            ->with('social_type', 'article')
            ->with('license', $license)
            ->with('navbar_items', $navbar_items)
            ->with('sidebar_content', $sidebar_content)
            ->with('page_content', $content)
            ->with('page_breadcrumbs', $breadcrumbs)
            ->with('page_slug', $slug)
            ->with('page_title', $page_title)
            ->with('page_revision', $revision)
            ->with('page_last_edit_date', $last_edit_date)
            ->with('page_last_edit_days_since', $last_edit_days_since)
            ->with('page_tags', $tags)
            ->with('page_category', $category);
    }
}
