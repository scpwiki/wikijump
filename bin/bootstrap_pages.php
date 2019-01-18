#!/usr/bin/env php
<?php

chdir(dirname(__FILE__));
require ('../php/setup.php');
Ozone::init();

$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey(1);
$page_facade = new Wikidot_Facade_Page($user);

$dirs = $argv;
array_shift($dirs);

foreach ($dirs as $dir) {
    foreach (ls('../' . $dir, '*.page') as $file) {
        echo "Saving $dir/$file\n";
        $source = file('../' . $dir . '/' . $file);
        $title = array_shift($source);
        $page_facade->save(array(
            'site' => basename($dir),
            'page' => str_replace('.', ':', preg_replace('/.page$/', '', $file)),
            'title' => $title,
            'source' => implode('', $source),
        ));
    }
}

