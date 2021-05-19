<?php

namespace Wikidot\Actions;
use Illuminate\Support\Facades\Hash;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\OzoneEmail;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;
use Wikidot\DB\Profile;
use Wikidot\DB\SitePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\Duplicator;
use Wikidot\Utils\FriendlyCaptchaHandler;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Models\User;

class CreateAccountStep2Action extends SmartyAction
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

        $errors = [];

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

            foreach (config('wikijump.forbidden_usernames') as $regex) {
                if (preg_match($regex, $unixName) > 0) {
                    $errors['name'] = _('Account creation failed: Username is blocked from registration.');
                }
            }

            // check if user does not exist
            $u = User::where('unix_name', $unixified)->count();
            if ($u > 0) {
                $errors['name'] = _("Account creation failed: A user with this screen name (or very similar) already exists.");
            }
        }

        // now check email
        if (filter_var($email, FILTER_VALIDATE_EMAIL, FILTER_FLAG_EMAIL_UNICODE) == false) {
            $errors['email'] = _("Account creation failed: Invalid email address.");
        } else {
            // check if email is unique
            $u = User::WhereRaw('lower(email) = ?', strtolower($email))->count();
            if ($u > 0) {
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
        if (!$lang) {
            $lang = env('DEFAULT_LANGUAGE', 'en');
        }

        // captcha
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

        $u = User::whereRaw('lower(email) = ?', strtolower($email))->count();

        if ($u > 0) {
            $runData->resetSession();
            throw new ProcessException(_("Account creation failed: A user with this email already exists."), "user_exists");
        }

        $unixified = WDStringUtils::toUnixName($name);
        $u = User::where('unix_name', $unixified)->count();

        if ($u > 0) {
            $runData->resetSession();
            throw new ProcessException(_("Account creation failed: A user with this name (or very similar) already exists."), "user_exists");
        }

        // add new user!!!

        $nuser = new User();
        $nuser->username = $name;
        $nuser->email = $email;
        $nuser->password = Hash::make($password);
        $nuser->unix_name = $unixified;
        $nuser->language = $lang;
        $nuser->save();

        $db->commit();

        /* Handle originalUrl. */
        $originalUrl = $runData->sessionGet('loginOriginalUrl') ?? null;
        if ($originalUrl) {
            $runData->ajaxResponseAdd('originalUrl', $originalUrl);
            if ($runData->sessionGet('loginOriginalUrlForce')) {
                $runData->ajaxResponseAdd('originalUrlForce', true);
            }
        }
        // reset session etc.
        $runData->resetSession();
        $runData->getSession()->setUserId($nuser->id);
        setcookie("welcome", $nuser->id, time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
        setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), null, "/");
    }

    public function cancelEvent($runData)
    {
        // reset session etc.
        $runData->resetSession();
    }
}
