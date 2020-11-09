<?php





/**
 * Path manager.
 *
 */
class PathManager{

	public static function ozonePhpCoreFile($fileName){
		return OZONE_ROOT."/php/core/".$fileName;
	}

	public static function ozonePhpServiceFile($fileName){
		return OZONE_ROOT."/php/resource/".$fileName;
	}

	public static function ozonePhpServiceAutoloadDir(){
		return OZONE_ROOT."/php/template_services/autoload/";
	}

	public static function ozoneApplicationPhpServiceAutoloadDir(){
		return WIKIJUMP_ROOT."/php/template_services/autoload/";
	}

	public static function ozonePhpServiceOnDemandDir(){
		return OZONE_ROOT."/php/template_services/ondemand/";
	}

	public static function ozoneApplicationPhpServiceOnDemandDir(){
		return WIKIJUMP_ROOT."/php/template_services/ondemand/";
	}

	public static function smartyDir(){
		return 	WIKIJUMP_ROOT."/lib/smarty/libs/";
	}

	public static function smartyOzonePluginDir(){
		return 	OZONE_ROOT."/php/smarty_plugins/";
	}

	public static function smartyPluginDir(){
		return 	WIKIJUMP_ROOT."/lib/smarty/libs/plugins/";
	}

	public static function smartyApplicationPluginDir(){
		return 	WIKIJUMP_ROOT."/php/smarty_plugins/";
	}

	public static function smartyCompileDir(){
			return 	WIKIJUMP_ROOT."/tmp/smarty_templates_c/";
	}

	public static function smartyCacheDir(){
			return 	WIKIJUMP_ROOT."/tmp/smarty_cache/";
	}

	public static function smartyMacroTemplateDir(){
			return 	WIKIJUMP_ROOT."/tmp/smarty_macro_templates/";
	}

	public static function navigationTemplate($name){

		return WIKIJUMP_ROOT."/templates/navigations/".$name.".tpl";
	}

	public static function navigationTemplateDir(){

		return WIKIJUMP_ROOT."/templates/navigations/";
	}

	public static function layoutTemplate($name){
		return 	WIKIJUMP_ROOT."/templates/layouts/".$name.".tpl";
	}

	public static function screenTemplate($name){
		str_replace(',', '/', $name);
		return 	WIKIJUMP_ROOT."/templates/screens/".$name.".tpl";
	}

	public static function templateDir(){
		return 	WIKIJUMP_ROOT."/templates/";
	}

	public static function screenClass($className){
		$className = str_replace(',', '/', $className);
		return 	WIKIJUMP_ROOT."/php/screens/".$className.".php";
	}

	public static function screenClassDir(){
		return 	WIKIJUMP_ROOT."/php/screens/";
	}

	public static function actionClass($className){
		return 	WIKIJUMP_ROOT."/php/actions/".$className.".php";
	}

	public static function moduleTemplate($name){
		return 	WIKIJUMP_ROOT."/templates/modules/".$name.".tpl";
	}

	public static function moduleClass($className){
		return 	WIKIJUMP_ROOT."/php/modules/".$className.".php";
	}

	public static function moduleClassDir(){
		return 	WIKIJUMP_ROOT."/php/modules/";
	}

	public static function macroDir(){
		return 	WIKIJUMP_ROOT."/templates/macros/";
	}

	public static function macroFile($name){
		return 	WIKIJUMP_ROOT."/templates/macros/".$name.".tpl";
	}

	public static function dbClass($className){
		return 	WIKIJUMP_ROOT."/php/db/".$className.".php";
	}

	public static function formSpecFile($formName){
		return 	WIKIJUMP_ROOT."/forms/".$formName."-form.xml";
	}

	public static function listSpecFile($listName){
		return 	WIKIJUMP_ROOT."/forms/".$listName."-list.xml";
	}

	public static function externalLibFile($fileName){
		return WIKIJUMP_ROOT.'/lib/'.$fileName;
	}

	public static function emailTemplate($templateName){
		return 	WIKIJUMP_ROOT."/templates/emails/".$templateName.".tpl";
	}

	public static function fileUploadDir(){
			return 	WIKIJUMP_ROOT."/tmp/file_uploads/";
	}

	public static function messagesDir(){
		return WIKIJUMP_ROOT."/messages/";
	}
}
