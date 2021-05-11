<?php

namespace Wikidot\Actions\Wiki;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\JSONService;
use Ozone\Framework\ODate;
use Ozone\Framework\OzoneEmail;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\EmailInvitationPeer;
use Wikidot\DB\EmailInvitation;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;

class UserInvitationAction extends SmartyAction
{

    public function perform($runData)
    {
    }

    public function sendEmailInvitationsEvent($runData)
    {
        $pl = $runData->getParameterList();
        $user = $runData->getUser();
        $site = $runData->getTemp("site");

        // is user allowed to send invitations?
        $siteSettings = $site->getSettings();
        $sendingEnabled = $siteSettings->getAllowMembersInvite();
        if (!$sendingEnabled) {
            throw new ProcessException(_("Users are not allowed to send invitations to this Wiki."));
        }

        if (!$user) {
            throw new ProcessException(_("You are not logged in."));
        }
        // check if a member
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $site->getSiteId());
        $mem = MemberPeer::instance()->selectOne($c);
        if (!$mem) {
            throw new ProcessException(_("Only members of this Wiki are allowed to send invitations."));
        }

        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $addresses = $json->decode($pl->getParameterValue("addresses"));

        $message = $pl->getParameterValue("message");
        // check if data is valid

        if (count($addresses) > 20) {
            throw new ProcessException(_("You should not send more than 20 invitations at once."));
        }

        foreach ($addresses as $address) {
            $email = trim($address[0]);
            $name = trim($address[1]);
            if (!preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) || strlen8($email)>70 || strlen($email) == 0) {
                throw new ProcessException(sprintf(_('Email "%s" is not valid.'), htmlspecialchars($email)), "bad_email");
            }

            if (preg_match(';://;', $name) || preg_match(';\.www;i', $name) || strlen($name)>50 || strlen($name) == 0) {
                throw new ProcessException(sprintf(_('Recipient\'s name "%s" is not valid.'), htmlspecialchars($name)), "bad_name");
            }

            //check if "email" is not already a member of this site...
            $q = " SELECT * FROM member, users WHERE member.site_id='".$site->getSiteId()."' AND users.email='".db_escape_string($email)."' AND member.user_id = users.id LIMIT 1";
            $c = new Criteria();
            $c->setExplicitQuery($q);
            $m = MemberPeer::instance()->selectOne($c);
            if ($m) {
                throw new ProcessException(sprintf(_('User with the email address "%s" is already a member of this Site. Remove them from the list and send invitations again.'), htmlspecialchars($email)), 'aleady_member');
            }

            // check if not sent already to this address.
            $c = new Criteria();
            $c->add("email", $email);
            $c->add("site_id", $site->getSiteId());
            $ii = EmailInvitationPeer::instance()->selectOne($c);

            if ($ii) {
                throw new ProcessException(sprintf(_('User with the email address "%s" has been already invited to this Site. Remove them from the list and send invitations again. If you want to resend an invitation please rather look at the history of sent invitations.'), htmlspecialchars($email)), 'aleady_member');
            }
        }

        if (preg_match(';://;', $message) || preg_match(';www\.;i', $message)) {
            throw new ProcessException(_('The message should not contain any links to websites.'), "bad_message");
        }
        if ($message != "" && strlen($message)>1000) {
            throw new ProcessException(_('The message seems to be too long. Max 1000 characters are allowed.'), "bad_message");
        }

        // now prepare invitation and send!

        $db = Database::connection();

        foreach ($addresses as $address) {
            $email = trim($address[0]);
            $name = trim($address[1]);
            $db->begin(); // each invitation makes a separate transaction

            $hash = substr(md5($name.$email).time(), 0, 20);

            $inv = new EmailInvitation();
            $inv->setHash($hash);
            $inv->setEmail($email);
            $inv->setName($name);
            $inv->setUserId($user->id);
            $inv->setSiteId($site->getSiteId());
            $inv->setMessage($message);
            $inv->setDate(new ODate());
            if ($address[2]) {
                $inv->setToContacts(true);
            }

            // prepare and send email

            $oe = new OzoneEmail();
            $oe->addAddress($email);
            $oe->setSubject(sprintf(_("[%s] %s invites you to join!"), GlobalProperties::$SERVICE_NAME, $user->username));
            $oe->contextAdd('user', $user);
            $oe->contextAdd('hash', $hash);
            $oe->contextAdd("site", $site);
            $oe->contextAdd("message", $message);
            $oe->contextAdd('name', $name);

            $oe->setBodyTemplate('MembershipEmailInvitation');

            if (!$oe->Send()) {
                $inv->setDelivered(false);
            } else {
                $inv->setDelivered(true);
            }

            $inv->save();

            $db->commit();
        }
    }

    public function deleteEmailInvitationEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        $user = $runData->getUser();

        $invitationId = $pl->getParameterValue("invitationId");

        $c = new Criteria();
        $c->add("invitation_id", $invitationId);
        $c->add("site_id", $site->getSiteId());

        $inv = EmailInvitationPeer::instance()->selectOne($c);

        if (!$inv) {
            throw new ProcessException(_("Invitation could not be found."), "no_invitation");
        }
        if ($inv->getUserId() != $user->id) {
            throw new ProcessException(_("This invitation does not seem to be sent by you..."));
        }

        // delete now
        EmailInvitationPeer::instance()->deleteByPrimaryKey($invitationId);
    }

    public function resendEmailInvitationEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $invitationId = $pl->getParameterValue("invitationId");

        $message2 = trim($pl->getParameterValue("message"));

        $c = new Criteria();
        $c->add("invitation_id", $invitationId);
        $c->add("site_id", $site->getSiteId());

        $inv = EmailInvitationPeer::instance()->selectOne($c);

        if (!$inv) {
            throw new ProcessException(_("Invitation could not be found."), "no_invitation");
        }

        if ($inv->getAttempts()>=3) {
            throw new ProcessException(_("You cannot send more than 3 copies of the invitation."));
        }

        if ($message2 == "") {
            throw new ProcessException(_('Message should not be empty'));
        }

        if (preg_match(';://;', $message2) || preg_match(';\.www;i', $message2)) {
            throw new ProcessException(_('The message should not contain any links to websites.'), "bad_message");
        }
        if ($message2 != "" && strlen($message2)>1000) {
            throw new ProcessException(_('The message seems to be too long. Max 1000 characters are allowed.'), "bad_message");
        }

        $db = Database::connection();
        $db->begin();

        // prepare and send email
        $user = $runData->getUser();

        $oe = new OzoneEmail();
        $oe->addAddress($inv->getEmail());
        $oe->setSubject(sprintf(_("[%s] %s invites you to join! (reminder)"), GlobalProperties::$SERVICE_NAME, $user->username));
        $oe->contextAdd('user', $user);
        $oe->contextAdd('hash', $inv->getHash());
        $oe->contextAdd("site", $site);
        $oe->contextAdd("message", $inv->getMessage());
        $oe->contextAdd("message2", $message2);
        $oe->contextAdd('name', $inv->getName());

        $oe->setBodyTemplate('MembershipEmailInvitation');

        $res = $oe->send();

        if (!$res) {
            throw new ProcessException("Email to this recipient could not be sent for some reason.");
        }
        $inv->setAttempts($inv->getAttempts()+1);
        $inv->save();
        $db->commit();
    }
}
