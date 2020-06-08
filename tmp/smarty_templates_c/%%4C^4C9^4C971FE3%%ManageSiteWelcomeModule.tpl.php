<?php /* Smarty version 2.6.7, created on 2008-12-06 17:49:50
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteWelcomeModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/managesite/ManageSiteWelcomeModule.tpl', 1, false),)), $this); ?>
<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Welcome to the site manager<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>!</h1>

<p>
	<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>This is the administrative panel for your Site.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>

	<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>But remember: <b>With Great Power Comes Great Responsibility</b>.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
</p>

<?php if ($this->_tpl_vars['tips']): ?>

	<h2><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>A few tips for your Site<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h2>

	<ul>
		<?php if ($this->_tpl_vars['tips']['forum']): ?>
			<li>
				<strong>Start a forum</strong><br/>
				The simplest way to add interactivity to your Site is to start a discussion forum.
				Click on <em>Forum &amp; discussion</em> &gt;&gt; <em>Settings</em> to learn more.<br/>
			</li>
		<?php endif; ?>
		<?php if ($this->_tpl_vars['tips']['tags']): ?>
			<li>
				<strong>Use site tags</strong><br/>
				If your Site already has some interesting content you should describe your Site by appropriate Tags
				- it will be easier to find for others. Go to
				<a href="javascript:;" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-general')">General settings</a>.
			</li>
		<?php endif; ?>
		<?php if ($this->_tpl_vars['tips']['invite']): ?>
			<li>
				<strong><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Invite your friends or coworkers!<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></strong><br/>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>It is easy to invite new people to join your site as members
				(if you need members at all).
				You can just send them emails with special invitations.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?><br/>
				<a href="javascript:;" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-email-invitations')"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Send email invitations now!<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>.
			</li>
			<li>
				Or even better - <a href="javascript:;" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-users-email-invitations')">you can allow your members to invite their friends</a> if you want the
				community to grow quickly!
			</li>
		<?php endif; ?>

	</ul>

<?php endif; ?>

<h2>Useful links</h2>

<ul>
	<li>
		<a href="<?php echo $this->_tpl_vars['URL_DOCS']; ?>
" target="_blank">Help &amp; Documentation</a> if you want to learn more or need help.
	</li>
</ul>