<?php
use DB\ContactPeer;

class AccountBackContactsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        // get all contacts
        $c = new Criteria();
        $c->add("contact.target_user_id", $user->getUserId());
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $contacts = ContactPeer::instance()->select($c);

        $runData->contextAdd("contacts", $contacts);
    }
}
