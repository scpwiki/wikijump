<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\Modules\PageRate\PageRateWidgetModule;

/**
 * Object Model mapped Class.
 *
 */
class Category extends CategoryBase
{

    public function getLicenseText()
    {
        if ($this->getName() === '_default') {
            if ($this->getLicenseId() == 1) {
                return $this->getLicenseOther();
            } else {
                $license = LicensePeer::instance()->selectById($this->getLicenseId());
                return $license->getDescription();
            }
        } else {
            if ($this->getLicenseDefault()) {
                // get default license (for the '_default' category
                $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
                return $dc->getLicenseText();
            } else {
                if ($this->getLicenseId() == 1) {
                    return $this->getLicenseOther();
                } else {
                    $license = LicensePeer::instance()->selectById($this->getLicenseId());
                    return $license->getDescription();
                }
            }
        }
    }

    public function getTopPage()
    {
        if ($this->getName() === '_default') {
            $pageName = $this->getTopBarPageName();
        } else {
            if ($this->getNavDefault()) {
                // get default category
                $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
                $pageName = $dc->getTopBarPageName();
            } else {
                $pageName = $this->getTopBarPageName();
            }
        }
        // now GET this page
        $page = PagePeer::instance()->selectByName($this->getSiteId(), $pageName);
        return $page;
    }

    public function getSidePage()
    {
        if ($this->getName() === '_default') {
            $pageName = $this->getSideBarPageName();
        } else {
            if ($this->getNavDefault()) {
                // get default category
                $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
                $pageName = $dc->getSideBarPageName();
            } else {
                $pageName = $this->getSideBarPageName();
            }
        }
        // now GET this page
        $page = PagePeer::instance()->selectByName($this->getSiteId(), $pageName);
        return $page;
    }

    public function getTheme()
    {
        if ($this->getExternalTheme()) {
            $theme = $this->getExternalTheme();
            if ($this->getName() !== '_default') {
                if ($this->getThemeDefault()) {
                    $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
                    $theme = $dc->getTheme();
                } else {
                    $theme = ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
                }
            }
            return $theme;
        }


        if ($this->getName() === '_default') {
            $theme = ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
        } else {
            if ($this->getThemeDefault()) {
                $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
                $theme = $dc->getTheme();
            } else {
                $theme = ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
            }
        }
        return $theme;
    }

    protected function getExternalTheme()
    {
        if (!$this->getThemeExternalUrl()) {
            return null;
        }
        $t = new Theme();
        $t->setExternalUrl($this->getThemeExternalUrl());
        /* Get base theme. */
        $c = new Criteria();
        $c->add('name', 'Base');
        $c->add('custom', false);
        $baseTheme = ThemePeer::instance()->selectOne($c);
        $t->setExtendsThemeId($baseTheme->getThemeId());
        $t->setThemeId($baseTheme->getThemeId()); // needed sometime
        return $t;
    }

    public function getPermissionString()
    {
        if ($this->getName() === '_default' || !$this->getPermissionsDefault()) {
            $ps = $this->getPermissions();
        } else {
            $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
            $ps = $dc->getPermissions();
        }
        return $ps;
    }

    public function getShowDiscuss()
    {
        $ppd = $this->getPerPageDiscussion();
        if ($ppd === null && $this->getName() !== '_default') {
            $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
            $ppd = $dc->getPerPageDiscussion();
        }
        if ($ppd === null) {
            $ppd = false;
        }
        return $ppd;
    }

    public function getRatingString()
    {
        $ppd = $this->getRating();
        if (strpos($ppd, 'e') === false && strpos($ppd, 'd') === false && $this->getName() !== '_default') {
            $dc = CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
            $ppd = $dc->getRating();
        }
        if ($ppd === null) {
            $ppd = 'd';
        }
        return $ppd;
    }

    /**
     * If the 'e' character is present in this category's `rating` table, return
     * true. Otherwise, return false.
     *
     * @return bool
     * @see PageRateWidgetModule::build()
     */
    public function getRatingEnabled() : bool
    {
        $s = $this->getRating();
        return strpos($s, 'e') !== false;
    }

    public function getRatingEnabledEff()
    {
        $s = $this->getRatingString();
        if (strpos($s, 'e') !== false) {
            return true;
        } elseif (strpos($s, 'd') !== false) {
            return false;
        } else {
            return null;
        }
    }

    public function getRatingType()
    {
        $s = $this->getRatingString();
        /*
         * P: Plus Only
         * M: Plus or Minus
         * Z: Plus, Zero, or Minus
         * X: Plus or Zero
         * S: Stars, 1-5
         *
         */
        preg_match('/(P|M|S|X|Z)/', $s, $m);
        $m = $m[0];
        if (!$m) {
            $m = 'P';
        }
        return $m;
    }

    public function getRatingBy()
    {
        $s = $this->getRatingString();
        if (strpos($s, 'm') !== false) {
            return 'm';
        } else {
            return 'r';
        }
    }

    public function getRatingVisible()
    {
        $s = $this->getRatingString();
        if (strpos($s, 'v')!== false) {
            return 'v';
        } else {
            return 'a';
        }
    }

    public function getTemplatePage()
    {
        $name = $this->getName();
        if ($name == '_default') {
            $name = '_template';
        } else {
            $name = $name . ':_template';
        }
        return PagePeer::instance()->selectByName($this->getSiteId(), $name);
    }

    public function save()
    {
        $key = 'category..'.$this->getSiteId().'..'.$this->getName();
        Cache::forget($key);
        $key = 'categorybyid..'.$this->getSiteId().'..'.$this->getCategoryId();
        Cache::forget($key);

        if ($this->getPerPageDiscussion() === null && $this->getName() == '_default') {
            $this->setPerPageDiscussion(false);
        }
        parent::save();
    }
}
