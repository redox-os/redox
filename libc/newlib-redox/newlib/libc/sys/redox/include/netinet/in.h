#ifndef _NETINET_IN_H_
#define _NETINET_IN_H_
/* Standard well-defined IP protocols.  */
enum {
    IPPROTO_IP = 0,    /* Dummy protocol for TCP.  */
#define IPPROTO_IP      IPPROTO_IP
    IPPROTO_ICMP = 1,      /* Internet Control Message Protocol.  */
#define IPPROTO_ICMP        IPPROTO_ICMP
    IPPROTO_TCP = 6,       /* Transmission Control Protocol.  */
#define IPPROTO_TCP     IPPROTO_TCP
    IPPROTO_UDP = 17,      /* User Datagram Protocol.  */
#define IPPROTO_UDP     IPPROTO_UDP
    IPPROTO_RAW = 255,     /* Raw IP packets.  */
#define IPPROTO_RAW     IPPROTO_RAW
    IPPROTO_MAX
};
#endif
