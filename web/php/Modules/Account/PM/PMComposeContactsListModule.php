<?php

namespace Wikidot\Modules\Account\PM;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ContactPeer;
use Wikidot\Utils\AccountBaseModule;

class PMComposeContactsListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        // get all contacts
        $c = new Criteria();
        $c->add("contact.user_id", $user->id);
        $c->addJoin("target_user_id", "users.id");
        $c->addOrderAscending("users.username");

        $contacts = ContactPeer::instance()->select($c);

        // avatar uri
        foreach ($contacts as &$co) {
            $userId = $co->getTargetUserId();
            $co->setTemp("avatarUri", '/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a16.png');
        }

        $runData->contextAdd("contacts", $contacts);
    }
}
