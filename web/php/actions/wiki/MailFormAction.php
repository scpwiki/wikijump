<?php
use DB\PagePeer;

class MailFormAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function sendFormEvent($runData)
    {
        $pl = $runData->getParameterList();
        $values = $pl->getParameterValue("formdata");

        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);

        $values = $json->decode($values);

        $site = $runData->getTemp("site");

        $fkey = trim($pl->getParameterValue("formdef"));

        $data = DatabaseStorage::instance()->get($fkey);

        if (!$data) {
            throw new ProcessException(_("No form definition found."));
        }

        $fields = $data['fields'];
        $email = $data['email'];
        $title = $data['title'];
        $format = strtolower(trim($data['format']));

        if (!in_array($format, array('csv'))) {
            $format = null;
        }

        // parse and validate!

        $errors = array();

        foreach ($fields as &$field) {
            $name = $field['name'];
            $value = $values[$field['name']];
            $field['value'] = $value;

            // check if need to validate. any rules?

            // first, if select, cannot be empty
            if ($field['type'] == "select") {
                if (!$value) {
                    $errors[$name] = _('Please select an option');
                    continue;
                }
            }

            if ($field['rules'] && is_array($field['rules'])) {
                foreach ($field['rules'] as $ruleName => $ruleValue) {
                    switch ($ruleName) {
                        case 'required':
                            if ($value=="") {
                                $errors[$name] = _('Please enter this information');
                                break 2;
                            }
                            break;
                        case 'minLength':
                            if (strlen8($value)<$ruleValue) {
                                $errors[$name] = _('Value is too short');
                                break 2;
                            }
                            break;
                        case 'maxLength':
                            if (strlen8($value)>$ruleValue) {
                                $errors[$name] = _('Value is too long');
                                break 2;
                            }
                            break;
                        case 'match':
                            if (!preg_match($ruleValue, $value)) {
                                $errors[$name] = _('Value is not valid');
                                break 2;
                            }
                            break;
                        case 'number':
                            if (!is_numeric($value)) {
                                $errors[$name] = _('Value is not numeric');
                                break 2;
                            }
                            break;
                        case 'minValue':
                            if (!is_numeric($value) || 1*$value<1*$ruleValue) {
                                $errors[$name] = _('Value is too small');
                                break 2;
                            }
                            break;
                        case 'maxValue':
                            if (!is_numeric($value) || 1*$value>1*$ruleValue) {
                                $errors[$name] = _('Value is too large');
                                break 2;
                            }
                            break;
                    }
                }
            }

            // fix checkboxes
            if ($field['type'] == "checkbox") {
                if (!$value) {
                    $field['value'] = _('No');
                } else {
                    $field['value'] = _('Yes');
                }
            }
        }

        if (count($errors)) {
            // "sir, we have some errors here. shit."
            $runData->ajaxResponseAdd("errors", $errors);
            throw new ProcessException("Form errors.", "form_errors");
        }

        $title = $title?$title:sprintf(_("[%s] MailForm form data"), GlobalProperties::$SERVICE_NAME);

        $oe = new OzoneEmail();
        $oe->addAddress($email);
        $oe->setSubject($title);
        $oe->contextAdd('fields', $fields);
        $oe->contextAdd('values', $values);

        switch ($format) {
            case 'csv':
                $emailTemplate = 'wiki/mailform/MailFormCSV';
                // fix the values (escape)
                foreach ($fields as &$field) {
                    $value = $field['value'];
                    if (preg_match("/[,\"\n]/", $value)) {
                        $value = str_replace('"', '""', $value);
                        $value = '"'.$value.'"';
                        $field['value'] = $value;
                    }
                }
                break;
            default:
                $emailTemplate = 'wiki/mailform/MailForm';
                break;
        }

        $oe->setBodyTemplate($emailTemplate);

        if (!$oe->Send()) {
            throw new ProcessException(_("The form data could not be sent to the specified email address."), "email_failed");
        }

        // ok, is there any success page?

        $successPage = $data['successPage'];
        if ($successPage) {
            $successPage = WDStringUtils::toUnixName($successPage);
            $page = PagePeer::instance()->selectByName($site->getSiteId(), $successPage);
            if ($page) {
                $runData->ajaxResponseAdd("successPage", $successPage);
            }
        }

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }
}
