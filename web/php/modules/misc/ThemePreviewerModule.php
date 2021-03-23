<?php

namespace Wikidot\Modules\Misc;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ThemePeer;
use Wikidot\DB\Theme;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ThemePreviewerModule extends SmartyModule
{

    protected $processPage = true;

    private $themeId;
    private $theme;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();

        $themeId = $pl->getParameterValue('theme_id');
        if ($themeId === null) {
            $themeUrl = current($_GET); // BAD HACK!!!
            if ($themeUrl) {
                $theme = $this->getExternalTheme($themeUrl);
            } else {
                $page = $runData->getTemp("page");
                if ($page == null) {
                    throw new ProcessException(_("Not working in the preview mode. Not a preview mode? So it might be an error."));
                }
                $theme = $page->getCategory()->getTheme();
            }
        } else {
            $theme = ThemePeer::instance()->selectByPrimaryKey($themeId);
        }

        //$this->themeId = $themeId;

        if ($theme == null || $theme->getAbstract() == true ||
            ($theme->getCustom ==true && $theme->getSiteId() != $site->getSiteId())) {
            throw new ProcessException(_("Error selecting theme."));
        }

        $this->theme = $theme;

        $q = "SELECT * from theme WHERE " .
                "abstract = FALSE  " .
                "AND (custom = FALSE" .
                    " OR (custom = TRUE AND site_id='".$site->getSiteId()."' AND site_id !=1)" .
                ") " .
                "ORDER BY custom, sort_index, replace(name, '_', '00000000');";

        $c = new Criteria();
        $c->setExplicitQuery($q);
        $themes = ThemePeer::instance()->select($c);
        $runData->contextAdd("themes", $themes);

        $runData->contextAdd("currentTheme", $theme);
        $runData->contextAdd("noUi", $pl->getParameterValue('noUi'));
    }

    public function processPage($out, $runData)
    {

        $theme = $this->theme;
        $t = '';
        foreach ($theme->getStyleUrls() as $url) {
            $t .= "@import url($url);\n";
        }

        $out = preg_replace('/
            (@import url\(
                [^\)]*?style\.css(\?[0-9]+)?  # URL must be style.css
            \);\s*)+
            /sx', $t, $out, 1);

        return $out;
    }

    protected function getExternalTheme($url)
    {
        if (!$url) {
            return null;
        }
        $t = new Theme();
        $t->setExternalUrl($url);
        /* Get base theme. */
        $c = new Criteria();
        $c->add('name', 'Base');
        $c->add('custom', false);
        $baseTheme = ThemePeer::instance()->selectOne($c);
        $t->setExtendsThemeId($baseTheme->getThemeId());
        $t->setThemeId($baseTheme->getThemeId()); // needed sometime
        return $t;
    }
}
