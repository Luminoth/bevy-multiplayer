use internal::notifs::Notification;

#[allow(dead_code)]
pub fn notify(_recipient: impl AsRef<str>, _notification: Notification, _ttl: Option<u64>) {
    // post a notif and add it to the mailbox
}
