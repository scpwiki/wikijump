<?php

namespace Wikidot\Modules\Account;


use Illuminate\Support\Facades\Auth;
use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\CryptUtils;

class AccountModule extends AccountBaseModule
{

    protected $processPage = true;

    public function isAllowed($runData)
    {

        return true;
    }

    public function build($runData)
    {
        if (!$runData->getUser() && !Auth::user()) {
            header("HTTP/1.1 301 Moved Permanently");
            header("Location: " . route('login'));
            return;
        }

        $user = $runData->getUser();
        $runData->contextAdd("user", $user);

        $pl = $runData->getParameterList();
        $start = $pl->getParameterValue("start");
        if ($start) {
            $runData->contextAdd("start", $start);
        }
        $composeTo = $pl->getParameterValue("composeto");
        if ($composeTo) {
            $runData->contextAdd("composeTo", $composeTo);
        }
        $inboxMessage = $pl->getParameterValue("inboxmessage");
        if ($inboxMessage) {
            $runData->contextAdd("inboxMessage", $inboxMessage);
        }
        // put the key too
        $runData->contextAdd("rsaKey", CryptUtils::modulus());
        $this->extraJs[] = '/common--javascript/crypto/rsa.js';
    }

    public function processPage($out, $runData)
    {
        $out = preg_replace("/<div id=\"page-title\">(.*?)<\/div>/is", '', $out, 1);
        return $out;
    }
}
