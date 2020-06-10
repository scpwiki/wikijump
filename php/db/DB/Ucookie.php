<?php


namespace DB;


//please extend this class
class Ucookie extends UcookieBase {

    public function generate(Site $site, OzoneSession $session) {
        $key = md5(gmdate("c") . rand());

        $this->setSite($site);
        $this->setOzoneSession($session);
        $this->setUcookieId($key);
    }
}
