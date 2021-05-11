<?php

namespace Wikidot\Modules\Account\Contacts;




use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\ContactPeer;
use Wikidot\Utils\AccountBaseModule;

class AccountContactsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        $contacts = DB::table('contact')
            ->join('users', 'target_user_id', '=', 'users.id')
            ->where('contact.user_id', $user->id)
            ->orderBy('users.username')
            ->get();

        // get the list who contacts you back to display emails.
        $q = DB::table('contact')
            ->select('user_id')
            ->where('target_user_id', $user->id)
            ->get()
            ->toArray();

        if ($q) {
            foreach ($q as &$b) {
                $b = $b['user_id'];
            }
            foreach ($contacts as &$contact) {
                if (in_array($contact->getTargetUserId(), $q)) {
                    $contact->setTemp("showEmail", true);
                }
            }
        }
        if (!$q) {
            $q = null;
        }
        $runData->contextAdd("back", $q);
        $runData->contextAdd("countBack", is_countable($q) ? count($q) : null);

        $runData->contextAdd("contacts", $contacts);

        $maxContacts = 10;
        $runData->contextAdd("maxContacts", $maxContacts);
    }
}
