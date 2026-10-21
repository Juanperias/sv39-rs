use sv39::paging::PagingMode;
use sv39::satp::Satp;

#[test]
fn test_host_satp_encoding() {
    let satp = Satp {
        mode: PagingMode::Sv39,
        asid: 1,
        ppn: 0x80200,
    };
    assert_eq!((satp.encode() >> 60) & 0xF, 8);
}
