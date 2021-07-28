<?php

namespace Wikidot\Modules\CreateAccount;


use Ozone\Framework\Database\Criteria;
use Wikidot\Actions\CreateAccountStep2Action;
use Wikidot\DB\OzoneSessionPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class CreateAccountStep2Module extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() !== null) {
            throw new ProcessException(_("You are already logged in."), "logged_in");
        }
        $rstep = $runData->sessionGet("rstep");
        return true;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        /* Process differently when there is a sessionHash code in the URL. */

        $sessionHash = $pl->getParameterValue('rcode');
        if ($sessionHash) {
            //var_dump($sessionHash);
            /* Get the session. */
            $c = new Criteria();
            $c->add('user_id', null);
            $c->add("md5(session_id || 'someseed')", $sessionHash);
            $session = OzoneSessionPeer::instance()->selectOne($c);

            if ($session) {
                $runData->setSession($session);

                /* Handle originalUrl. */
                $originalUrl = $runData->sessionGet('loginOriginalUrl');
                if ($originalUrl) {
                    $runData->contextAdd('originalUrl', $originalUrl);
                    if ($runData->sessionGet('loginOriginalUrlForce')) {
                        $runData->contextAdd('originalUrlForce', true);
                    }
                }

                /* Complete the registration. */
                require_once(WIKIJUMP_ROOT . '/php/Actions/CreateAccountStep2Action.php');
                $action = new CreateAccountStep2Action();
                $action->finalizeEvent($runData, true);

                $runData->contextAdd('fromEmail', true);

                return;
            }
        }

        $evcode = $runData->sessionGet('evcode');
        if (!$evcode) {
            throw new ProcessException(_('Not within registration chain. <a href="/auth:newaccount">Click this</a> to start a new account.'));
        }
        $runData->contextAdd('evcode', $runData->sessionGet('evcode'));
        $runData->sessionAdd("rstep", 2);

        $data = $runData->sessionGet("ca_data");
        $email = $data['email'];
        $name = $data['name'];

        $runData->contextAdd("email", $email);
        $runData->contextAdd("name", $name);
    }
}
