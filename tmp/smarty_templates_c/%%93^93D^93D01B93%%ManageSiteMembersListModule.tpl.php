<?php /* Smarty version 2.6.7, created on 2009-01-04 19:48:28
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteMembersListModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'printuser', '/var/www/wikidot/templates/modules/managesite/ManageSiteMembersListModule.tpl', 6, false),)), $this); ?>
<h1>List of members</h1>

<div>
<?php if (count($_from = (array)$this->_tpl_vars['memberships'])):
    foreach ($_from as $this->_tpl_vars['membership']):
?>
	<?php $this->assign('user', $this->_tpl_vars['membership']->getUser()); ?>
	<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['user'],'image' => 'yes'), $this);?>
 
	
	<div style="padding-left: 20px">
		member since:  <span class="odate"><?php echo $this->_tpl_vars['membership']->getDateJoined()->getTimestamp(); ?>
|%e %b %Y, %H:%M %Z|agohover</span>
		(<a href="javascript:;" onclick="if($('mem-options-<?php echo $this->_tpl_vars['user']->getUserId(); ?>
').style.display=='none') $('mem-options-<?php echo $this->_tpl_vars['user']->getUserId(); ?>
').style.display = 'block'; else $('mem-options-<?php echo $this->_tpl_vars['user']->getUserId(); ?>
').style.display = 'none';">options</a>)
		
		<div id="mem-options-<?php echo $this->_tpl_vars['user']->getUserId(); ?>
" style="display: none">
			
			<a href="javascript:;" onclick="removeUser(<?php echo $this->_tpl_vars['user']->getUserId(); ?>
, '<?php echo $this->_tpl_vars['user']->getNickName(); ?>
')">remove</a>
			| <a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteMembersListModule.listeners.removeAndBan(<?php echo $this->_tpl_vars['user']->getUserId(); ?>
, '<?php echo $this->_tpl_vars['user']->getNickName(); ?>
')">remove &amp; ban</a>
			| <a href="javascript:;" onclick="toModerators(<?php echo $this->_tpl_vars['user']->getUserId(); ?>
)">to moderators</a>
			| <a href="javascript:;" onclick="toAdmins(<?php echo $this->_tpl_vars['user']->getUserId(); ?>
, '<?php echo $this->_tpl_vars['user']->getNickName(); ?>
')">to admins</a>
			
		</div>
	</div>
<?php endforeach; endif; unset($_from); ?>
</div>

<div style="display: none" id="remove-user-dialog">
	<h1>Remove user?</h1>
	<p>
		Are you sure you want to remove user <b>%%USER_NAME%%</b> from the members?
	</p>
	<p>
		If the user is also an administrator/moderator of this site his privileges will be lost too.
	</p>
</div>
<div style="display: none" id="remove-ban-user-dialog">
	<h1>Remove and ban user?</h1>
	<p>
		Are you sure you want to remove user <b>%%USER_NAME%%</b> from the members <u>and</u>
		ban from accessing the site in the future?
	</p>
	<p>
		If the user is also an administrator/moderator of this site his privileges will be lost too.
	</p>
</div>