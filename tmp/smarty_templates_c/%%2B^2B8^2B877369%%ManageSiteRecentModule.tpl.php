<?php /* Smarty version 2.6.7, created on 2009-01-04 19:50:37
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteRecentModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'module', '/var/www/wikidot/templates/modules/managesite/ManageSiteRecentModule.tpl', 3, false),)), $this); ?>
<h1>Recent page changes</h1>

<?php echo smarty_function_module(array('name' => "changes/SiteChangesModule"), $this);?>