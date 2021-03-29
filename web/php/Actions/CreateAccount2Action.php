<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\OzoneEmail;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\DB\OzoneUser;
use Wikidot\DB\Profile;
use Wikidot\DB\UserSettings;
use Wikidot\DB\SitePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\Duplicator;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDStringUtils;

class CreateAccount2Action extends SmartyAction
{

    protected static $EVCODE_SEED = 'someseed';

    public function perform($runData)
    {
    }

    public function acceptRulesEvent($runData)
    {
        $accept = $runData->getParameterList()->getParameterValue("acceptrules");
        if (!$accept) {
            throw new ProcessException(_("You must accept the Terms of Service before proceeding."), "must_accept");
        }
    }

    public function step0Event($runData)
    {

        // do it manually. change of rules.
        $pl = $runData->getParameterList();
        $name = $pl->getParameterValue("name");
        $email = $pl->getParameterValue("email");
        $password = $pl->getParameterValue("password");
        $password2 = $pl->getParameterValue("password2");
        $captcha = $pl->getParameterValue("frc-captcha-solution");

        // validate now.

        $errors = array();

        //name
        $unixified = WDStringUtils::toUnixName($name);
        if (strlen($name)<2) {
            $errors['name'] = _("Account creation failed: Username too short. Minimum 3 characters.");
        } elseif (strlen8($name)>20) {
            $errors['name'] = _("Account creation failed: Username too long. Maximum 20 characters.");
        } elseif (preg_match('/^[ _a-zA-Z0-9-\!#\$%\^\*\(\)]+$/', $name) == 0) {
            $errors['name'] = _("Account creation failed: Accepted characters in usernames are (a-z, A-Z, 0-9, !, #, $, %, ^, *, (, ), _, and space.");
        } elseif (strlen($unixified)<2) {
            $errors['name'] = _("Account creation failed: Username needs at least 2 non-special characters.");
        } else {
            //handle forbidden names
            $unixName = WDStringUtils::toUnixName($name);

            $forbiddenUnixNames = explode("\n", file_get_contents(WIKIJUMP_ROOT.'/conf/forbidden_user_names.conf'));
            foreach ($forbiddenUnixNames as $f) {
                if (preg_match($f, $unixName) >0) {
                    $errors['name'] = _('Account creation failed: Username is blocked from registration.');
                }
            }

            // check if user does not exist
            $c = new Criteria();
            $c->add("unix_name", $unixified);
            $u = OzoneUserPeer::instance()->selectOne($c);
            if ($u != null) {
                $errors['name'] = _("Account creation failed: A user with this screen name (or very similar) already exists.");
            }
        }

        // now check email
        if (filter_var($email, FILTER_VALIDATE_EMAIL, FILTER_FLAG_EMAIL_UNICODE) == false) {
            $errors['email'] = _("Account creation failed: Invalid email address.");
        } else {
            // check if email is unique
            $c = new Criteria();
            $c->add("lower(email)", strtolower($email));
            $u = OzoneUserPeer::instance()->selectOne($c);
            if ($u != null) {
                $errors['email'] = _("Account creation failed: A user with this email already exists.");
            }
        }

        // check password
        if (strlen8($password)<8) {
            $errors['password'] = _("Account creation failed: Password minimum is 8 characters.");
        } elseif (strlen8($password)>256) {
            $errors['password'] = _("Account creation failed: Maximum password length is 256 characters to avoid denial of service.");
        } elseif ($password2 != $password) {
            $errors['password2'] = _("Account creation failed: Passwords are not identical.");
        }

        // check language
        $lang = $pl->getParameterValue("language");
        if ($lang !== "pl" && $lang !== "en") {
            $errors['language'] = _("Please select your preferred language.");
        }

        // captcha
        // verify reslt with FriendlyCaptcha
        $captchaValid = FriendlyCaptchaHandler::verifySolution($captcha);
        if (!$captchaValid) {
            $errors['captcha'] = _("Account creation failed: CAPTCHA was invalid.");
        }

        if (!$pl->getParameterValue("tos")) {
            $errors['tos'] = _("Account creation failed: Please read and agree to the Terms of Service.");
        }

        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        // store data in the session

        $data = array(
            'name' => $name,
            'email' => $email,
            'password' => $password ,
            'language' =>$lang
        );

        $runData->sessionAdd("ca_data", $data);

        // send email HERE:

        $data = $runData->sessionGet("ca_data");
        $email = $data['email'];
        $name = $data['name'];

        //generate the email verification code

        $evcode = $runData->sessionGet('evcode');
        if (!$evcode) {
            srand((double)microtime()*1000000);
            $string = md5(rand(0, 9999));
            $evcode = substr($string, 2, 9);
        }

        //send a confirmation email to the user.
        $oe = new OzoneEmail();
        $oe->addAddress($email);
        $oe->setSubject(sprintf(_("%s - email verification"), GlobalProperties::$SERVICE_NAME));
        $oe->contextAdd('name', $name);
        $oe->contextAdd('email', $email);
        $oe->contextAdd('evcode', $evcode);
        $oe->contextAdd('sessionHash', md5($runData->getSession()->getSessionId() . self::$EVCODE_SEED));

        $oe->setBodyTemplate('RegistrationEmailVerification');

        if (!$oe->Send()) {
            throw new ProcessException(_("The email cannot be sent to this address."), "email_failed");
        }
        $runData->sessionAdd('evcode', $evcode);
    }

    public function sendEmailVerEvent($runData)
    {

        $data = $runData->sessionGet("ca_data");
        $email = $data['email'];
        $name = $data['name'];

        //generate the email verification code

        $evcode = $runData->sessionGet('evcode');
        if ($evcode == null) {
            srand((double)microtime()*1000000);
            $string = md5(rand(0, 9999));
            $evcode = substr($string, 2, 6);
        }

        //send a confirmation email to the user.
        $oe = new OzoneEmail();
        $oe->addAddress($email);
        $oe->setSubject(sprintf(_("%s- email verification"), GlobalProperties::$SERVICE_NAME));
        $oe->contextAdd('name', $name);
        $oe->contextAdd('email', $email);
        $oe->contextAdd('evcode', $evcode);

        $oe->setBodyTemplate('RegistrationEmailVerification');

        if (!$oe->Send()) {
            throw new ProcessException(_("The email cannot be sent to this address."), "email_failed");
        }
        $runData->sessionAdd('evcode', $evcode);
    }

    public function finalizeEvent($runData, $skipEvcode = false)
    {
        // get the form data
        $pl = $runData->getParameterList();

        if (!$skipEvcode) {
            $evcode = $pl->getParameterValue("evcode", "AMODULE");

            //check if the email vercode is correct
            $evcode2 = $runData->sessionGet('evcode');
            if ($evcode !== $evcode2) {
                throw new ProcessException(_("Invalid email verification code."), "invalid_code");
            }
        }

        $data = $runData->sessionGet("ca_data");

        $name = $data['name'];
        $email = $data['email'];
        $password = $data['password'];
        $lang = $data['language'];

        $db = Database::connection();
        $db->begin();

        // check again if email and nick are not duplicate!

        $c = new Criteria();
        $c->add("lower(email)", strtolower($email));
        $u = OzoneUserPeer::instance()->selectOne($c);
        if ($u != null) {
            $runData->resetSession();
            throw new ProcessException(_("Account creation failed: A user with this email already exists."), "user_exists");
        }

        $unixified = WDStringUtils::toUnixName($name);
        $c = new Criteria();
        $c->add("unix_name", $unixified);
        $u = OzoneUserPeer::instance()->selectOne($c);
        if ($u != null) {
            $runData->resetSession();
            throw new ProcessException(_("Account creation failed: A user with this name (or very similar) already exists."), "user_exists");
        }

        // add new user!!!

        $nuser = new OzoneUser();
        /* email as the username!!! */
        $nuser->setName($email);
        $nuser->setEmail($email);
        $nuser->setPassword($password);

        $nuser->setNickName($name);
        $nuser->setUnixName($unixified);

        $nuser->setLanguage($lang);

        $date = new ODate();
        $nuser->setRegisteredDate($date);
        $nuser->setLastLogin($date);

        $nuser->save();

        // profile

        $profile = new Profile();
        $profile->setUserId($nuser->getUserId());
        $profile->save();

        $us = new UserSettings();
        $us->setUserId($nuser->getUserId());
        $us->save();

        // profile page

        $c = new Criteria();
        $c->add("unix_name", "profiles");
        $nsite = SitePeer::instance()->selectOne($c);
        $ncategory = CategoryPeer::instance()->selectByName('profile', $nsite->getSiteId());

        $dup = new Duplicator;
        $dup->setOwner($nuser);

        $dup->duplicatePage(
            PagePeer::instance()->selectByName($nsite->getSiteId(), 'template:profile'),
            $nsite,
            $ncategory,
            'profile:'.$nuser->getUnixName()
        );

        $page = PagePeer::instance()->selectByName($nsite->getSiteId(), 'profile:'.$nuser->getUnixName());

        $ou = new Outdater();
        $ou->pageEvent('new_page', $page);

        $db->commit();

        /* Handle originalUrl. */
        $originalUrl = $runData->sessionGet('loginOriginalUrl');
        if ($originalUrl) {
            $runData->ajaxResponseAdd('originalUrl', $originalUrl);
            if ($runData->sessionGet('loginOriginalUrlForce')) {
                $runData->ajaxResponseAdd('originalUrlForce', true);
            }
        }
        // reset session etc.
        $runData->resetSession();
        $runData->getSession()->setUserId($nuser->getUserId());
        setcookie("welcome", $nuser->getUserId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
        setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), null, "/");
    }

    public function cancelEvent($runData)
    {
        // reset session etc.
        $runData->resetSession();
    }
}
