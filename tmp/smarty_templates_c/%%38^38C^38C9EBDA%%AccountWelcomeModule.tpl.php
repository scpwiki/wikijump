<?php /* Smarty version 2.6.7, created on 2008-12-06 17:44:13
         compiled from /var/www/wikidot/templates/modules/account/AccountWelcomeModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/account/AccountWelcomeModule.tpl', 1, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/account/AccountWelcomeModule.tpl', 1, false),)), $this); ?>
<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Welcome<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>, <?php echo ((is_array($_tmp=$this->_tpl_vars['user']->getNickName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
!</h1>

<?php if ($this->_tpl_vars['tips']): ?>

	<h2><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>A few tips just for you<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:</h2>

	<ul>
		<?php if ($this->_tpl_vars['tips']['avatar']): ?>
			<li>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>You have not uploaded your buddy icon (avatar) yet. So now you are using a default<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
				avatar: <div style="text-align:center">
				<img src="/common--images/avatars/default/a48.png" alt=""/>.
				</div>
				<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Go to <em>my profile</em><?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> >> <em><a href="javascript:;" onclick="OZONE.ajax.requestModule('account/profile/APAvatarModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>my buddy icon<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a></em> <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>to upload your very own image<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>.
			</li>
		<?php endif; ?>
	</ul>


<?php endif; ?>

<div style="float:left; width: 44%; padding: 0 2%">
<h2>Shortcuts:</h2>
<ul>
	<li><a href="/new-site">Get a new wiki</a></li>
	<li><a href="http://<?php echo $this->_tpl_vars['URL_HOST']; ?>
">Go to main page</a></li>

</ul>


</div>
