<?php

namespace Wikidot\Modules\Account\Contacts;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ContactPeer;
use Wikidot\Utils\AccountBaseModule;

class AccountBackContactsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        // get all contacts
        $c = new Criteria();
        $c->add("contact.target_user_id", $user->id);
        $c->addJoin("user_id", "users.id");
        $c->addOrderAscending("users.username");

        $contacts = ContactPeer::instance()->select($c);

        $runData->contextAdd("contacts", $contacts);
    }
}
