<?php

namespace Wikidot\DB;


use Wikidot\Utils\ProcessException;

/**
 * Object Model Class.
 *
 */
class Theme extends ThemeBase
{

    protected $_external;

    public function getStyleUrls()
    {

        if ($this->getExtendsThemeId()) {
            // get parent theme
            $parent = ThemePeer::instance()->selectByPrimaryKey($this->getExtendsThemeId());
            if ($parent == null) {
                throw new ProcessException("FATAL ERROR: Theme not found.");
            }
            $files = $parent->getStyleUrls();
        } else {
            $files = array();
        }

        $files[] = $this->getStyleUrl();
        return $files;
    }

    /**
     * Returns url of the style associated with this theme.
     */
    public function getStyleUrl()
    {
        if ($this->getCustom()) {
            return "/local--theme/" . $this->getUnixName() . "/style.css?" . $this->getRevisionNumber();
        } elseif ($this->_external) {
            return $this->_external;
        } else {
            return "/common--theme/" . $this->getUnixName() . "/css/style.css?" . $this->getRevisionNumber();
        }
    }

    public function getThemePreview()
    {
        return ThemePreviewPeer::instance()->selectByPrimaryKey($this->getThemeId());
    }

    public function setExternalUrl($url)
    {
        $this->_external = $url;
    }
}
