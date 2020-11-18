<?php
use DB\SitePeer;
use DB\PagePeer;

class Backuper
{

    private $siteId;

    private $backupSource = true;
    private $backupFiles = true;
    private $rand;

    public function setConfig($sb)
    {
        $this->backupSource = $sb->getBackupSource();
        $this->backupFiles = $sb->getBackupFiles();
        $this->siteId = $sb->getSiteId();
    }

    public function backup()
    {

        $site = SitePeer::instance()->selectByPrimaryKey($this->siteId);
        if (!$site) {
            throw new ProcessException(_("Site cannot be found"));
        }

        // prepare working directory

        $wdir = WIKIJUMP_ROOT.'/tmp/sitebackups/'.$site->getUnixName().'/work';
        @exec('rm -r '.$wdir.' &> /dev/null');
        mkdirfull($wdir);

        if ($this->backupSource) {
            mkdirfull($wdir.'/source');
            // iterate through pages

            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $pages = PagePeer::instance()->select($c);

            foreach ($pages as $page) {
                $source = $page->getCurrentRevision()->getSourceText();
                $filename = $page->getUnixName().'.txt';
                $filename = str_replace(':', '_', $filename);
                file_put_contents($wdir.'/source/'.$filename, $source);
            }
        }
        if ($this->backupFiles) {
            mkdirfull($wdir.'/files');
        /*  $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $pages = DB_PagePeer::instance()->select($c);

            foreach($pages as $page){
                // get the files
                $c = new Criteria();
                $c->add("page_id", $page->getPageId());
                $files = DB_FilePeer::instance()->select($c);
                if(count($files)>0){
                    $path = $wdir.'/files/'.$page->getUnixName();
                    mkdirfull($path);
                    foreach($files as $file){
                        copy($file->getFilePath(), $path);
                    }
                }
            }   */

            $path0 = WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName().'/files/';
            $cmd = "cp -r ".$path0.'*'.' '.$wdir.'/files/'.' &> /dev/null';

            @exec($cmd);
            // fix colon:
            $dirstmp = ls($wdir.'/files/', '*:*');
            foreach ($dirstmp as $dd) {
                @rename($wdir.'/files/'.$dd, $wdir.'/files/'.str_replace(':', '_', $dd));
            }
        }

        // zip the content
        $cmd = 'cd '.$wdir.' && zip -r backup *';
        exec($cmd);

        $zipfile = $wdir.'/backup.zip';
        if (!file_exists($zipfile)) {
            throw new ProcessException("Error creating backup.");
        }
        // dest dir
        @exec('rm -r '.WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup/'.' &> /dev/null');
        $rand = md5(rand(10000, 99999).time());
        $ddir = WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup/'.$rand.'/';
        mkdirfull($ddir);

        copy($zipfile, $ddir.'backup.zip');

        // clear the working dir
        @exec('rm -r '.escapeshellarg($wdir).' &> /dev/null');

        $this->rand = $rand;
    }

    public function setSiteId($siteId)
    {
        $this->siteId = $siteId;
    }

    public function setBackupSource($val)
    {
        $this->backupSource = $val;
    }
    public function setBackupFiles($val)
    {
        $this->backupFiles = $val;
    }

    public function getRand()
    {
        return $this->rand;
    }
}
