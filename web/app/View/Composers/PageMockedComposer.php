<?php

declare(strict_types=1);

namespace Wikijump\View\Composers;

use Illuminate\View\View;
use Faker;
use Wikijump\Services\License\License;
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

        $content = $this->generateContent();
        $sidebar_content = $this->generateSidebarContent();
        $navbar_items = $this->generateNavbarItems();
        $license = $this->getRandomLicense();
        $breadcrumbs = $this->generateBreadcrumbs();

        $site_name = $f->company;
        $page_title = $f->streetName;
        $revision = $f->randomNumber(3);
        $timestamp = $f->dateTimeThisYear->getTimestamp();
        $tags = $f->unique()->words(10);
        $title = "$page_title | $site_name";
        $social_title = $page_title;
        $category = $f->word;

        $view
            ->with('title', $title)
            ->with('social_title', $social_title)
            ->with('social_type', 'article')
            ->with('license', $license)
            ->with('navbar_items', $navbar_items)
            ->with('sidebar_content', $sidebar_content)
            ->with('page_content', $content)
            ->with('page_breadcrumbs', $breadcrumbs)
            ->with('page_title', $page_title)
            ->with('page_revision', $revision)
            ->with('page_last_edit_timestamp', $timestamp)
            ->with('page_tags', $tags)
            ->with('page_category', $category);
    }

    /** Generates content as a random assortment of paragraph elements. */
    private function generateContent(int $min = 10, int $max = 100): string {
        $f = $this->faker;
        $content = '';
        for ($i = 0; $i < $f->numberBetween($min, $max); $i++) {
            if ($f->boolean) {
                $content .= $f->paragraph;
            } else {
                $content .= '<p>' . $f->paragraph . '</p>';
            }
        }

        $content = "<wj-body class=\"wj-body\">$content</wj-body>";

        return $content;
    }

    /**
     * Generates sidebar content as an assortment of anchors and headers.
     * Might return null randomly, to indicate that the sidebar should not be shown.
     */
    private function generateSidebarContent(int $min = 1, int $max = 10): ?string {
        $f = $this->faker;
        $sidebar_content = null;
        if ($f->boolean) {
            $sidebar_content = '';
            for ($i = 0; $i < $f->numberBetween($min, $max); $i++) {
                if ($f->boolean) {
                    $sidebar_content .= '<h1>' . $f->streetName . '</h1>';
                } else {
                    $sidebar_content .=
                        '<div><a href="' . $f->url . '">' . $f->streetName . '</a></div>';
                }
            }

            $sidebar_content = "<wj-body class=\"wj-body\">$sidebar_content</wj-body>";
        }

        return $sidebar_content;
    }

    /** Generates a random navbar items array. */
    private function generateNavbarItems(int $min = 1, int $max = 5): array {
        $f = $this->faker;
        $navbar_items = [];
        for ($i = 0; $i < $f->numberBetween($min, $max); $i++) {
            $title = $f->streetName;
            if ($f->boolean) {
                $link = $f->streetName;
                $navbar_items[$link] = [];
                for ($j = 0; $j < $f->numberBetween($min, $max); $j++) {
                    $navbar_items[$link][$f->streetName] = $f->url;
                }
            } else {
                $navbar_items[$title] = $f->url;
            }
        }
        return $navbar_items;
    }

    /** Generates a random breadcrumbs array. */
    private function generateBreadcrumbs(int $min = 1, int $max = 3): array {
        $f = $this->faker;
        $breadcrumbs = [];
        if ($f->boolean) {
            for ($i = 0; $i < $f->numberBetween($min, $max); $i++) {
                $breadcrumbs[] = [
                    'title' => $f->streetName,
                    'url' => $f->url,
                ];
            }
        }
        return $breadcrumbs;
    }

    /** Returns a random license from the config. */
    private function getRandomLicense(): License {
        $list = config('licenses.raw');
        $id = $list[array_rand($list)]['id'];
        return LicenseMapping::get($id);
    }
}
