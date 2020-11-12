<?php
use DB\ContactPeer;

class AccountContactsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        // get all contacts
        $c = new Criteria();
        $c->add("contact.user_id", $user->getUserId());
        $c->addJoin("target_user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");

        $contacts = ContactPeer::instance()->select($c);

        if (true || count($contacts) > 0) {
            // get the list who contacts you back to display emails.
            // by query
            $q = "SELECT user_id FROM contact WHERE target_user_id='" . $user->getUserId() . "'";
            $db = Database::connection();
            $res = $db->query($q);
            $back = $res->fetchAll();

            if ($back) {
                foreach ($back as &$b) {
                    $b = $b['user_id'];
                }
                foreach ($contacts as &$contact) {
                    if (in_array($contact->getTargetUserId(), $back)) {
                        $contact->setTemp("showEmail", true);
                    }
                }
            }
            if (!$back) {
                $back = null;
            }
            $runData->contextAdd("back", $back);
            $runData->contextAdd("countBack", count($back));
        }

        $runData->contextAdd("contacts", $contacts);

        $maxContacts = 10;
        $runData->contextAdd("maxContacts", $maxContacts);
    }
}
