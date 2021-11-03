<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Wikijump\Common\Enum;

final class AbuseReportType extends Enum
{
    const PAGE = 'page';
    const USER = 'user';
}
