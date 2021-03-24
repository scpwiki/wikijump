<?php

namespace Wikidot\Utils;

/**
 * Module controlls permissions, resolves module names etc.
 */
class ModuleManager
{

    private static $instance;

    private $wikiConfig;
    private $ajaxConfig;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new ModuleManager();
        }
        return self::$instance;
    }

    /**
     * Takes module name used in Wiki source and returns template name
     */
    public function resolveWikiModuleName($name)
    {
        if ($this->wikiConfig == null) {
            $this->loadWikiConfig();
        }
        $row = $this->wikiConfig[$name];
        return $row['template'];
    }

    private function loadWikiConfig()
    {
        /* find all files with module configs */
        $fs = glob(WIKIJUMP_ROOT.'/conf/wiki_modules/*.conf');
        $cont = '';
        foreach ($fs as $f) {
            $c = file_get_contents($f);
            $c = preg_replace('/^#.*?$/sm', '', $c);
            $c = trim($c);
            $cont .= "\n" . $c;
        }
        $cont = trim($cont);
        $m1 = explode("\n", $cont);
        $stor = array();
        foreach ($m1 as $m) {
            $m3 = explode(" ", $m);
            $stor[$m3[0]] = array('name' => $m3[0], 'template' =>$m3[1], 'permissions' => $m3[2]);
        }
        $this->wikiConfig = $stor;
    }

    public function canWikiUseModule($siteName, $moduleName)
    {
        if ($this->wikiConfig == null) {
            $this->loadWikiConfig();
        }

        $row = $this->wikiConfig[$moduleName];

        if ($row == null) {
            return false;
        }
        if ($row['permissions'] == null) {
            return true;
        }

        $sites = explode(",", $row['permissions']);
        if (in_array($siteName, $sites)) {
            return true;
        } else {
            return false;
        }
    }
}
