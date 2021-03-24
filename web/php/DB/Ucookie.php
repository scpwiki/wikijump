<?php

namespace Wikidot\DB;




//please extend this Class
class Ucookie extends UcookieBase
{

    public function generate(Site $site, OzoneSession $session)
    {
        $key = md5(gmdate("c") . rand());

        $this->setSite($site);
        $this->setOzoneSession($session);
        $this->setUcookieId($key);
    }
}
