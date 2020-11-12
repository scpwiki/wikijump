<?php
use DB\OzoneUserPeer;
use DB\ContactPeer;
use DB\Contact;

class ContactsAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if ($userId == null || $userId <1) {
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

        $targetUser = OzoneUserPeer::instance()->selectByPrimaryKey($targetUserId);

        $user = $runData->getUser();

        if ($targetUser == null) {
            throw new ProcessException(_("User cannot be found."), "no_user");
        }

        if ($targetUserId == $user->getUserId()) {
            throw new ProcessException(_("Is there any point in adding yourself to your contact list?"), "not_yourself");
        }

        $db = Database::connection();
        $db->begin();

        // check if already contacted
        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $c->add("target_user_id", $targetUserId);

        $contact = ContactPeer::instance()->selectOne($c);
        if ($contact) {
            throw new ProcessException(_("This user is already in your contacts."), "already_contact");
        }

        // count contacts
        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $count = ContactPeer::instance()->selectCount($c);
        if ($count>=1000) {
            throw new ProcessException(_("Sorry, at this moment you cannot add more than 1000 contacts.", "max_reached"));
        }

        //...

        $contact = new Contact();
        $contact->setUserId($user->getUserId());
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
        $c->add("user_id", $user->getUserId());
        $c->add("target_user_id", $targetUserId);

        ContactPeer::instance()->delete($c);
    }
}
