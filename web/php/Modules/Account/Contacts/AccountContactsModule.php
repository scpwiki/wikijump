<?php

namespace Wikidot\Modules\Account\Contacts;




use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\ContactPeer;
use Wikidot\Utils\AccountBaseModule;

class AccountContactsModule extends AccountBaseModule
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

        // get the list who contacts you back to display emails.
        // by query
        $q = "SELECT user_id FROM contact WHERE target_user_id='" . $user->id . "'";
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
        $runData->contextAdd("countBack", is_countable($back) ? count($back) : null);

        $runData->contextAdd("contacts", $contacts);

        $maxContacts = 10;
        $runData->contextAdd("maxContacts", $maxContacts);
    }
}
