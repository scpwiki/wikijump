<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SmartyAction;

use Wikidot\DB\ContactPeer;
use Wikidot\DB\Contact;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

class ContactsAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if(!$userId) {
            throw new WDPermissionException(_("Not allowed. You should login first."));
        }
        return true;
    }

    public function perform($runData)
    {
    }

    public function addContactEvent($runData)
    {

        $pl = $runData->getParameterList();

        $targetUserId = $pl->getParameterValue("userId");

        $targetUser = User::find($targetUserId);

        $user = $runData->getUser();

        if ($targetUser == null) {
            throw new ProcessException(_("User cannot be found."), "no_user");
        }

        if ($targetUserId == $user->id) {
            throw new ProcessException(_("Is there any point in adding yourself to your contact list?"), "not_yourself");
        }

        $db = Database::connection();
        $db->begin();

        // check if already contacted
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("target_user_id", $targetUserId);

        $contact = ContactPeer::instance()->selectOne($c) ?? null;
        if ($contact) {
            throw new ProcessException(_("This user is already in your contacts."), "already_contact");
        }

        // count contacts
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $count = ContactPeer::instance()->selectCount($c);
        if ($count>=1000) {
            throw new ProcessException(_("Sorry, at this moment you cannot add more than 1000 contacts.", "max_reached"));
        }

        //...

        $contact = new Contact();
        $contact->setUserId($user->id);
        $contact->setTargetUserId($targetUserId);
        $contact->save();

        $db->commit();
    }

    public function removeContactEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();
        $targetUserId = $pl->getParameterValue("userId");

        if ($targetUserId == null) {
            throw new ProcessException(_("No user found."), "no_user");
        }

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("target_user_id", $targetUserId);

        ContactPeer::instance()->delete($c);
    }
}
