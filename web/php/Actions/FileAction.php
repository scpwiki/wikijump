<?php

namespace Wikidot\Actions;

use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\PagePeer;
use Wikidot\DB\FilePeer;
use Wikidot\DB\File;
use Wikidot\Utils\FileHelper;
use Wikidot\Utils\FileMime;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;

class FileAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function checkFileExistsEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        $pageId = $pl->getParameterValue("pageId");
        $fileName = trim($pl->getParameterValue("filename"));

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Problem selecting destination page."), "no_page");
        }

        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('attach_file', $user, $category, $page);

        // mangle filename to extract file only. does not have to be 100% fail-safe.
        $f = preg_split("/[\/\\\\]/", $fileName);
        $fileName = end($f);

        $c = new Criteria();
        $c->add("filename", $fileName);
        $c->add("site_id", $site->getSiteId());
        $c->add("page_id", $pageId);

        $file = FilePeer::instance()->selectOne($c);

        if ($file == null) {
            $runData->ajaxResponseAdd("exists", false);
        } else {
            $runData->ajaxResponseAdd("exists", true);
            $runData->setModuleTemplate("Files/FileUploadExistsWinModule");
            $runData->contextAdd("file", $file);
            $runData->contextAdd("destinationPage", $page);

            // check if has permissions... TODO!
            $hasPermission = true;
            try {
                WDPermissionManager::instance()->hasPagePermission('replace_file', $user, $category, $page);
                $overwritePermission = true;
            } catch (Exception $e) {
                $overwritePermission = false;
            }
            $runData->contextAdd("hasPermission", $overwritePermission);
        }
    }

    public function uploadFileEvent($runData)
    {

        try {
            // the event method will not use OZONE functionality for file processing but
            // rather a low-level approach.

            $status = "ok"; // status variable that will be passed to template

            $pl = $runData->getParameterList();
            $site = $runData->getTemp("site");
            $pageId = $pl->getParameterValue("page_id");
            $page = PagePeer::instance()->selectByPrimaryKey($pageId);
            if ($page == null || $page->getSiteId() != $site->getSiteId()) {
                $status = "error";
                $runData->contextAdd("status", $status);
                $runData->contextAdd("message", _("Page does not exist???"));
                return;
            }

            $category = $page->getCategory();
            // now check for permissions!!!
            $user = $runData->getUser();
            WDPermissionManager::instance()->hasPagePermission('attach_file', $user, $category, $page);

            $userId = $runData->getUserId() ?? null;
            if ($userId == null) {
                $userString = $runData->createIpString();
            }

            $file = $_FILES['userfile'];

            $comments = trim($pl->getParameterValue("comments"));
            $comments = substr($comments, 0, 110);

            if ($file['error'] ===2 || $file['error'] ===1) {
                $status = "size_error";
                $runData->contextAdd("status", $file['error']);
                $runData->contextAdd("message", _("Error uploading file - file size exceeds limit."));
                return;
            }

            if ($file['error'] ===3) {
                $status = "partial_error";
                $runData->contextAdd("status", $file['error']);
                $runData->contextAdd("message", _("Error uploading file - file only partially uploaded."));
                return;
            }

            if ($file['error'] ==4) {
                $status = "no_file";
                $runData->contextAdd("status", $file['error']);
                $runData->contextAdd("message", _("Error uploading file - no file uploaded."));
                return;
            }
            if ($file['error'] !=0) {
                $status = "other_error";
                $runData->contextAdd("status", $file['error']);
                $runData->contextAdd("message", _("Error uploading file - no file uploaded."));
                return;
            }

            if ($file['size'] == 0) {
                $status = "zero_size";
                $runData->contextAdd("status", $status);
                $runData->contextAdd("message", _("Error uploading file - the file has 0 bytes size."));
                return;
            }

            if (!is_uploaded_file($file['tmp_name'])) {
                $status = "invalid_file";
                $runData->contextAdd("status", $status);
                $runData->contextAdd("message", _("Error uploading file - invalid file."));

                return;
            }

            $totalSize = FileHelper::totalSiteFilesSize($site->getSiteId());
            $allowed = $site->getSettings()->getFileStorageSize();

            $maxUpload = min($allowed - $totalSize, $site->getSettings()->getMaxUploadFileSize());
            if ($file['size'] > $maxUpload) {
                $status = "too_big";
                $runData->contextAdd("status", $status);
                $runData->contextAdd("message", _("Error uploading file - file size exceeds limit."));
                return;
            }

            // check if destination file exists!
            $destinationFilename = $pl->getParameterValue("dfilename");
            if ($destinationFilename === "" || $destinationFilename == null) {
                // use the original name
                $destinationFilename = $file['name'];
            }

            $c = new Criteria();
            $c->add("filename", $destinationFilename);
            $c->add("site_id", $site->getSiteId());
            $c->add("page_id", $pageId);

            $conflictFiles = FilePeer::instance()->select($c);
            if (count($conflictFiles)>0) {
                // file already exists!!!
                try {
                    WDPermissionManager::instance()->hasPagePermission('replace_file', $user, $category, $page);
                    $overwritePermission = true;
                } catch (Exception $e) {
                    $overwritePermission = false;
                }

                if ($pl->getParameterValue("force") && $overwritePermission) {
                    FilePeer::instance()->delete($c);
                } else {
                    $status = "file_exists";
                    $runData->contextAdd("status", $status);
                    $runData->contextAdd("message", _("Error uploading file - file by that name already exists."));
                    return;
                }
            }

            // determine mime type using file cmd
            $fdesc = FileMime::description($file['tmp_name']);
            $fmime = FileMime::mime($file['tmp_name']);

            $uploadDir = $site->getLocalFilesPath()."/files/".$page->getUnixName();
            mkdirfull($uploadDir);

            $dest = $uploadDir."/".$destinationFilename;

            move_uploaded_file($file['tmp_name'], $dest);

            // check if image and resize

            // DO NOT RUN identify ON ALL FILES!!!!!!!!!!!!
            // OR limit the resources please
            $cmd = 'identify '.escapeshellarg($dest);
            $res = exec_time($cmd, 8, $out);
            if ($res) {
                // is at least "imageable" - can have thumbnails
                // resized images dir
                $resizedDir = $site->getLocalFilesPath() . "/resized-images/".$page->getUnixName().
                        '/'.$destinationFilename;
                mkdirfull($resizedDir);

                $hasResized = $this->resizeImages($resizedDir, $dest);
            }

            $db = Database::connection();
            $db->begin();

            // if successfull create new file object and insert into database.
            $f = new File();
            $f->setPageId($pageId);
            $f->setFilename($destinationFilename);
            $f->setSize($file['size']);
            $f->setDateAdded(new ODate());
            if ($userId) {
                $f->setUserId($userId);
            } else {
                $f->setUserId(User::ANONYMOUS_USER);
                $f->setUserString($userString);
            }

            $f->setSiteId($site->getSiteId());

            $f->setComment($comments);

            $f->setMimetype($fmime);
            $f->setDescription($fdesc);

            $f->setHasResized($hasResized);

            $sdesc = explode(",", $fdesc);
            $sdesc = $sdesc[0];
            $f->setDescriptionShort($sdesc);

            $f->save();
            // create a new revision
            $revision = $page->getCurrentRevision();
            $revision->setNew(true);
            $revision->setRevisionId(null);
            $revision->resetFlags();
            $revision->setFlagFile(true);

            $revision->setComments("Uploaded file \"$destinationFilename\".");

            if ($userId) {
                $revision->setUserId($userId);
                $page->setLastEditUserId($userId);
            } else {
                $revision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $revision->setUserString($userString);
                $page->setLastEditUserString($userString);
            }

            $revision->setRevisionNumber($revision->getRevisionNumber() +1);
            $now = new ODate();
            $revision->setDateLastEdited($now);

            $revision->save();
            $page->setRevisionId($revision->getRevisionId());
            $page->setDateLastEdited($now);
            $page->setRevisionNumber($revision->getRevisionNumber());
            $page->save();

            // in case there is a gallery plugin or an image pointing
            // to the file - simpy recompile the page

            $od = new Outdater();
            $od->pageEvent('file_change', $page);

            $db->commit();
            $runData->contextAdd("status", "ok");
        } catch (Exception $e) {
            $status = "not_ok";
            $runData->contextAdd("status", $status);
            $runData->contextAdd("message", _("Error uploading file."));
            $db->rollback();
        }
    }

    public function renameFileEvent($runData)
    {

        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");

        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();

        $file = FilePeer::instance()->selectByPrimaryKey($fileId);
        $page = PagePeer::instance()->selectByPrimaryKey($file->getPageId());

        if ($file == null || $file->getSiteId() != $site->getSiteId() || $page==null) {
            throw new ProcessException(_("Error getting file data."), "file_error");
        }
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_page");
        }

        // check permissions
        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('rename_file', $user, $category, $page);

        $newName = trim($pl->getParameterValue("new_name"));

        if ($newName == null || $newName=='') {
            throw new ProcessException(_("No new name given."), "name_error");
        }
        if (strlen($newName)>90) {
            throw new ProcessException(_("New file name too long."), "name_error");
        }
        if ($newName === $file->getFilename()) {
            throw new ProcessException(_("New and old names are the same."), "name_error");
        }

        try {
            WDPermissionManager::instance()->hasPagePermission('replace_file', $user, $category, $page);
            $overwritePermission = true;
        } catch (Exception $e) {
            $overwritePermission = false;
        }

        // check if file exists with this name
        $force = $pl->getParameterValue("force");
        if ($force && $overwritePermission) {
            // delete any file by this name in the page
            $c = new Criteria();
            $c->add("page_id", $page->getPageId());
            $c->add("filename", $newName);
            $conflict = FilePeer::instance()->selectOne() ?? null;
            // delete from filesystem
            if ($conflict) {
                $cmd = "rm ".escapeshellarg($conflict->getFilePath());
                exec($cmd);
                // delete resized images if exist
                if ($conflict->getHasResized()) {
                    $cmd = "rm -r ".escapeshellarg($conflict->getResizedDir());
                    exec($cmd);
                }
            }
            FilePeer::instance()->delete($c);
        }
        // ok, move along. nothing to watch.
        $c = new Criteria();
        $c->add("page_id", $page->getPageId());
        $c->add("filename", $newName);

        $conflictFile = FilePeer::instance()->selectOne($c);
        if ($conflictFile != null) {
            // file already exists!!! ask what to do!
            $runData->contextAdd("newFile", $conflictFile);
            $runData->contextAdd("file", $file);
            $runData->setModuleTemplate("Files/FileRenameExistsWinModule");

            $runData->contextAdd("hasPermission", $overwritePermission);
            $db->commit();
            $runData->ajaxResponseAdd("status", "file_exists");
            return;
        }

        $oldPath = $file->getFilePath();
        $oldRDir = $file->getResizedDir();

        $oldName = $file->getFilename();
        $file->setFilename($newName);
        $file->save();

        $newPath = $file->getFilePath();
        $newRDir = $file->getResizedDir();

        // create a new revision
        $revision = $page->getCurrentRevision();
        $revision->setNew(true);
        $revision->setRevisionId(null);
        $revision->resetFlags();
        $revision->setFlagFile(true);
        $revision->setComments("File \"$oldName\" renamed to \"$newName\".");

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
        }
        if ($userId) {
            $revision->setUserId($userId);
            $page->setLastEditUserId($userId);
        } else {
            $revision->setUserId(User::ANONYMOUS_USER);
            $page->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $page->setLastEditUserString($userString);
        }
        $revision->setRevisionNumber($revision->getRevisionNumber() +1);
        $now = new ODate();
        $revision->setDateLastEdited($now);

        $revision->save();
        $page->setRevisionId($revision->getRevisionId());
        $page->setDateLastEdited($now);
        $page->setRevisionNumber($revision->getRevisionNumber());
        $page->save();
        if (rename("$oldPath", "$newPath") == false) {
            throw new ProcessException(_("Error moving files."), "error_moving");
        }
        if ($file->getHasResized()) {
            if (rename("$oldRDir", "$newRDir") == false) {
                throw new ProcessException(_("Error moving resized files."), "error_moving");
            }
        }

        $od = new Outdater();
        $od->pageEvent('file_change', $page);

        $db->commit();
    }

    public function moveFileEvent($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");
        $destinationPageName = $pl->getParameterValue("destination_page_name");
        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();
        $user = $runData->getUser();

        $file = FilePeer::instance()->selectByPrimaryKey($fileId);
        $page = PagePeer::instance()->selectByPrimaryKey($file->getPageId());

        if ($file == null || $file->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting file data."), "file_error");
        }
        if ($page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting file information."), "no_page");
        }

        $categoryFrom = $page->getCategory();
        // now check for permissions!!!
        WDPermissionManager::instance()->hasPagePermission('move_file', $user, $categoryFrom, $page);

        if ($destinationPageName == $page->getUnixName()) {
            throw new ProcessException(_("There is not point in moving the file to the same (current)  page..."), "no_destination");
        }

        $destinationPage = PagePeer::instance()->selectByName($site->getSiteId(), $destinationPageName);

        if ($destinationPage == null) {
            throw new ProcessException(_("Destination page does not exist."), "no_destination");
        }

        // check for permissions now to attach the file to a new page.
        $categoryTo = $destinationPage->getCategory();
        try {
            WDPermissionManager::instance()->hasPagePermission('attach_file', $user, $categoryTo, $destinationPage);
        } catch (Exception $e) {
            throw new ProcessException(_("No permission to add file to the specifed new page."), "no_destination_permission");
        }

        try {
            WDPermissionManager::instance()->hasPagePermission('replace_file', $user, $categoryTo, $destinationPage);
            $overwritePermission = true;
        } catch (Exception $e) {
            $overwritePermission = false;
        }

        // check if file exists in the destination page
        $force = $pl->getParameterValue("force");

        if ($force && $overwritePermission) {
            // delete any file by this name in the page
            $c = new Criteria();
            $c->add("page_id", $destinationPage->getPageId());
            $c->add("filename", $file->getFilename());
            FilePeer::instance()->delete($c);
        }
        $c = new Criteria();
        $c->add("page_id", $destinationPage->getPageId());
        $c->add("filename", $file->getFilename());
        $conflictFile = FilePeer::instance()->selectOne($c);
        if ($conflictFile != null) {
            // file already exists!!! ask what to do!
            // check permissions to overwrite?
            $runData->contextAdd("page", $page);
            $runData->contextAdd("destinationPage", $destinationPage);
            $runData->contextAdd("file", $file);
            $runData->setModuleTemplate("Files/FileMoveExistsWinModule");

            $runData->contextAdd("hasPermission", $overwritePermission);
            $runData->ajaxResponseAdd("status", "file_exists");

            $db->commit();

            return;
        }
        // ok, move along. nothing to watch.
        $oldPath = $file->getFilePath();
        $oldRDir = $file->getResizedDir();

        $file->setPageId($destinationPage->getPageId());
        $file->save();
        $newPath = $file->getFilePath();
        $newRDir = $file->getResizedDir();

        $dir = dirname($newPath);
        mkdirfull($dir);
        if (rename($oldPath, $newPath) == false) {
            throw new ProcessException(_("Error moving files."), "error_moving");
        }
        if ($file->getHasResized()) {
            $resizedDir = $site->getLocalFilesPath()."/resized-images/".$destinationPage->getUnixName();
            mkdirfull($resizedDir);
            if (rename("$oldRDir", "$newRDir") == false) {
                throw new ProcessException(_("Error moving resized files."), "error_moving");
            }
        }

        $runData->ajaxResponseAdd("moved", true);

        // create new revisions of $page and $destinationPage

        // create a new revision
        $revision = $page->getCurrentRevision();
        $revision->setNew(true);
        $revision->setRevisionId(null);
        $revision->resetFlags();
        $revision->setFlagFile(true);

        $revision->setRevisionNumber($revision->getRevisionNumber() +1);
        $now = new ODate();
        $revision->setDateLastEdited($now);

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
        }
        if ($userId) {
            $revision->setUserId($userId);
            $page->setLastEditUserId($userId);
        } else {
            $revision->setUserId(User::ANONYMOUS_USER);
            $page->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $page->setLastEditUserString($userString);
        }
        $revision->setComments('File "'.$file->getFilename().'" moved away to page "'.$destinationPage->getUnixName().'".');
        $revision->save();
        $page->setRevisionId($revision->getRevisionId());
        $page->setDateLastEdited($now);
        $page->setRevisionNumber($revision->getRevisionNumber());
        $page->save();

        // and destinationPage
        // create a new revision
        $revision = $destinationPage->getCurrentRevision();
        $revision->setNew(true);
        $revision->setRevisionId(null);
        $revision->resetFlags();
        $revision->setFlagFile(true);

        $revision->setRevisionNumber($revision->getRevisionNumber() +1);
        $revision->setDateLastEdited($now);

        if ($userId) {
            $revision->setUserId($userId);
            $destinationPage->setLastEditUserId($userId);
        } else {
            $revision->setUserId(User::ANONYMOUS_USER);
            $destinationPage->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $destinationPage->setLastEditUserString($userString);
        }
        $revision->setComments('File "'.$file->getFilename().'" moved from page "'.$page->getUnixName().'".');
        $revision->save();
        $destinationPage->setRevisionId($revision->getRevisionId());
        $destinationPage->setDateLastEdited($now);
        $destinationPage->setRevisionNumber($revision->getRevisionNumber());
        $destinationPage->save();

        $od = new Outdater();
        $od->pageEvent('file_change', $page);
        $od->pageEvent('file_change', $destinationPage);

        $db->commit();
    }

    public function deleteFileEvent($runData)
    {
        $pl = $runData->getParameterList();
        $fileId = $pl->getParameterValue("file_id");
        $destinationPageName = $pl->getParameterValue("destination_page_name");
        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();
        $file = FilePeer::instance()->selectByPrimaryKey($fileId);

        if ($file == null || $file->getSiteId() != $site->getSiteId()) {
            throw new ProcessException("File does not exist.", "no_file");
        }
        $page = PagePeer::instance()->selectByPrimaryKey($file->getPageId());
        if ($page == null) {
            throw new ProcessException(_("Page does not exist."), "no_page");
        }

        $category = $page->getCategory();
        // now check for permissions!!!
        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('delete_file', $user, $category, $page);

        // remove file! and create another revision too...
        @unlink($file->getFilePath());
        // delete resized images if exist
        if ($file->getHasResized()) {
            $cmd = "rm -r ".escapeshellarg($file->getResizedDir());
            exec($cmd);
        }
        FilePeer::instance()->deleteByPrimaryKey($file->getFileId());
        // create a new revision
        $revision = $page->getCurrentRevision();
        $revision->setNew(true);
        $revision->setRevisionId(null);
        $revision->resetFlags();
        $revision->setFlagFile(true);

        $revision->setRevisionNumber($revision->getRevisionNumber() +1);
        $now = new ODate();
        $revision->setDateLastEdited($now);

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
        }
        if ($userId) {
            $revision->setUserId($userId);
            $page->setLastEditUserId($userId);
        } else {
            $revision->setUserId(User::ANONYMOUS_USER);
            $page->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $page->setLastEditUserString($userString);
        }
        $revision->setComments('File "'.$file->getFilename().'" deleted.');

        $revision->save();
        $page->setRevisionId($revision->getRevisionId());
        $page->setDateLastEdited($now);
        $page->setRevisionNumber($revision->getRevisionNumber());
        $page->save();

        $od = new Outdater();
        $od->pageEvent('file_change', $page);

        $db->commit();
    }

    private function resizeImages($path, $filename)
    {

        // generate image paths
        $medium = escapeshellarg($path.'/medium.jpg');
        $small = escapeshellarg($path.'/small.jpg');
        $thumbnail = escapeshellarg($path.'/thumbnail.jpg');
        $square = escapeshellarg($path.'/square.jpg');

        $is = getimagesize($filename);
        if ($is[2] == 3 || $is[2] == 1) {
            $tmpfile = $path.'/tmpfile.png';
            $tmpfile_x = escapeshellarg($tmpfile);
            copy($filename, $tmpfile);
            $cmd = 'convert ' . $tmpfile_x . ' -background white -flatten ' . $tmpfile_x . ' 2>&1';
            exec_time($cmd, 8);
            $cmd = 'convert ' . $tmpfile_x . ' -resize \'500x500>\'  +profile \'*\'  ' . $medium . ' 2>&1';
        } else {
            $cmd = 'convert '.escapeshellarg($filename.'[0]').' -resize \'500x500>\'  +profile \'*\'  '.$medium.' 2>&1';
        }
        exec_time($cmd, 8);
        if (file_exists($tmpfile)) {
            unlink($tmpfile);
        }
        $cmd = 'convert '.$medium.' -resize \'240x240>\' -unsharp 0x1.0+1.0+0.10 '.$small.'';
        exec_time($cmd, 8);
        $cmd = 'convert '.$small.' -resize \'100x100>\' -unsharp 0x1.0+1.0+0.10 '.$thumbnail.'';
        exec_time($cmd, 8);
        // get  dimension
        if (!file_exists($path.'/small.jpg')) {
            return  false;
        }
        $size = getimagesize($path.'/small.jpg');
        $w = $size[0];
        $h = $size[1];
        if ($h>$w) {
            $ns = '75x'.$h*75.0/$w;
        } else {
            $ns = $w*75.0/$h.'x75';
        }
        $cmd = 'convert '.$small.' -resize '.$ns.'  '.$square.'';
        exec_time($cmd, 8);
        // and crop it!
        $cmd = 'convert '.$square.'  -gravity Center -crop 75x75+0+0 +repage '.$square.'';
        exec_time($cmd, 8);
        return true;
    }
}
