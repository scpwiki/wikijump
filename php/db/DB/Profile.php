<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class Profile extends ProfileBase
{

    public function getBirthdayDate($format = null)
    {
        if ($this->getBirthdayDay() == null) {
            return null;
        }

        $day = $this->getBirthdayDay();
        $month = $this->getBirthdayMonth();
        $year = $this->getBirthdayYear();

        $date = mktime(0, 0, 0, $month, $day, $year);
        if ($format == null) {
            $format = "%e %b %Y";
        }
        return strftime($format, $date);
    }
}
