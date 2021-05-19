<?php

namespace Wikidot\Actions;
use Illuminate\Support\Facades\Hash;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\OzoneEmail;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\PrivateUserBlockPeer;
use Wikidot\DB\PrivateUserBlock;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

class AccountSettingsAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if(!$userId) {
            throw new WDPermissionException(_("Not allowed. You should login first."));
        }
        return true;
    }

    public function perform($r)
    {
    }

    public function changePasswordEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();

        $oldPassword = $pl->getParameterValue("old_password");
        $newPassword1 = ($pl->getParameterValue("new_password1"));
        $newPassword2 = ($pl->getParameterValue("new_password2"));

        if (password_verify($oldPassword, $user->password) == false) {
            throw new ProcessException(_("Password reset failed: Your current password is incorrect."), "form_error");
        }
        if ($newPassword1 !== $newPassword2) {
            throw new ProcessException(_("Password reset failed: New passwords do not match."), "form_error");
        }
        if (strlen8($newPassword1)<8) {
            throw new ProcessException(_("Password reset failed: Minimum password length is 8 characters."), "form_error");
        }
        if (strlen8($newPassword1)>256) {
            throw new ProcessException(_("Password reset failed: Maximum password length is 256 characters to avoid denial of service."), "form_error");
        }

        // ok, change the password!!!
        $user->password = Hash::make($newPassword1);
        $user->save();
    }

    public function changeEmail1Event($runData)
    {
        $pl = $runData->getParameterList();

        $email = $pl->getParameterValue("email", "AMODULE");

        if ($email == null || $email == '') {
            throw new ProcessException(_("Email must be provided."), "no_email");
        }

        if (filter_var($email, FILTER_VALIDATE_EMAIL, FILTER_FLAG_EMAIL_UNICODE) == false) {
            throw new ProcessException(_("Valid email must be provided."), "no_email");
        }

        // check for users with the email
        $user = User::firstWhere('email', $email);

        if ($user !== null) {
            throw new ProcessException(_("An user with this email already exists. Emails must be unique."), "form_error");
        }

        // generate code
        srand((double)microtime()*1000000);
        $string = md5(rand(0, 9999));
        $evcode = substr($string, 2, 6);

        //send a confirmation email to the user.
        $oe = new OzoneEmail();
        $oe->addAddress($email);
        $oe->setSubject(sprintf(_("%s - email address change"), GlobalProperties::$SERVICE_NAME));
        $oe->contextAdd("user", $runData->getUser());
        $oe->contextAdd("email", $email);
        $oe->contextAdd('evcode', $evcode);

        $oe->setBodyTemplate('ChangeEmailVerification');

        if (!$oe->Send()) {
            throw new ProcessException(_("The email cannot be sent to this address."), "form_error");
        }

        $runData->sessionAdd("chevcode", $evcode);
        $runData->sessionAdd("ch-nemail", $email);
        $runData->contextAdd("email", $email);
    }

    public function changeEmail2Event($runData)
    {
        $pl = $runData->getParameterList();

        $evercode = $pl->getParameterValue("evercode");

        if ($evercode != $runData->sessionGet("chevcode")) {
            throw new ProcessException(_("The verification codes do not match."), "form_error");
        }
        $email = $runData->sessionGet("ch-nemail");
        $runData->sessionDel("ch-nemail");
        $runData->sessionDel("chevcode");

        $user = $runData->getUser();
        $user->email = $email;
        $user->save();

        $runData->contextAdd("email", $email);
    }

    public function saveReceiveInvitationsEvent($runData)
    {

        $pl = $runData->getParameterList();
        $receive = $pl->getParameterValue("receive") ?? null;
        if ($receive) {
            $receive = true;
        } else {
            $receive = false;
        }
        $user = User::find($runData->getUserId());
        $user->set(['receive_invitations' => $receive]);
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveReceiveMessagesEvent($runData)
    {

        $pl = $runData->getParameterList();
        $from = $pl->getParameterValue("from");

        if ($from !== "a" && $from !== "mf" && $from !=="f" && $from !== "n") {
            $from = "a";
        }

        $user = User::find($runData->getUserId());
        $user->set(['receive_pm' => $from]);
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function blockUserEvent($runData)
    {
        $pl = $runData->getParameterList();
        $userId = $pl->getParameterValue("userId");

        if ($userId == null || !is_numeric($userId)) {
            throw new ProcessException(_("Invalid user."), "no_user");
        }

        $user = User::find($userId);
        if ($user == null) {
            throw new ProcessException(_("Invalid user."), "no_user");
        }

        // check if already blocked
        $c = new Criteria();
        $c->add("user_id", $runData->getUserId());
        $c->add("blocked_user_id", $userId);
        $b = PrivateUserBlockPeer::instance()->selectOne($c) ?? null;
        if ($b) {
            throw new ProcessException(_("You already block this user."));
        }

        // check max
        $c = new Criteria();
        $c->add("user_id", $runData->getUserId());
        $blockCount = PrivateUserBlockPeer::instance()->selectCount($c);

        $maxBlocks = 30;

        if ($blockCount>$maxBlocks) {
            throw new ProcessException("Sorry, you can only block $maxBlocks users max.", "max_block");
        }

        if ($userId == $runData->getUserId()) {
            throw new ProcessException(_("What is the point in blocking yourself?"), "not_self");
        }

        $block = new PrivateUserBlock();
        $block->setUserId($runData->getUserId());
        $block->setBlockedUserId($userId);
        $block->save();
    }

    public function deleteBlockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $blockedUserId = $pl->getParameterValue("userId");
        $userId = $runData->getUserId();

        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->add("blocked_user_id", $blockedUserId);

        PrivateUserBlockPeer::instance()->delete($c);
    }

    public function saveReceiveDigestEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();

        $receive = (bool) $pl->getParameterValue("receive");

        $settings = $user->getSettings();
        if ($receive != $settings->getReceiveDigest()) {
            $settings->setReceiveDigest($receive);
            $settings->save();
        }

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveReceiveNewsletterEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();

        $receive = (bool) $pl->getParameterValue("receive");

        $settings = $user->getSettings();
        if ($receive != $settings->getReceiveNewsletter()) {
            $settings->setReceiveNewsletter($receive);
            $settings->save();
        }

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveLanguageEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();

        $lang = $pl->getParameterValue("language");

        if ($lang !== "pl" && $lang !=="en") {
            throw new ProcessException(_("Error selecting the language"));
        }

        $user->language = $lang;
        $user->save();

        $runData->ajaxResponseAdd("language", $lang);
    }
}
